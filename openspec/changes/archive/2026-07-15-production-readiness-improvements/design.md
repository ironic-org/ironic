## Context

Ironic is structured as a single Rust crate with all subsystems compiled via `#[path = "..."]` attributes from `src/lib.rs`. The workspace has only two members: the root crate and `crates/ironic-macros/`. Subsystem directories (`crates/ironic-*/`) are source folders, not separate crates. This means:

- All dependencies share a single `Cargo.toml`
- Feature flags for optional dependencies are defined at the root level
- There are no crate-boundary enforcements between subsystems

The audit revealed 12+ production gaps across observability, metrics, health checks, configuration, resilience, and developer experience. Current state: OTLP has zero dependencies wired, metrics use a global `Mutex<Vec<f64>>` that grows unboundedly, `/health` is static, rate limiting is in-memory only, there are no config profiles, no static file serving, no multipart, and no persistent session store.

## Goals / Non-Goals

**Goals:**
- Every P0 gap resolved: OTLP telemetry operational, lock-free metrics, composite health checks, drain timeout
- Every P1 gap resolved: Redis rate limiting, config profiles, per-endpoint metrics wired
- Every P2 gap resolved: static files, multipart uploads, Redis sessions, error backtraces
- Every P3 gap resolved: backpressure, OAuth2 callback, config hot reload, feature toggles
- All changes gated behind optional feature flags
- Documentation updated for every new subsystem page

**Non-Goals:**
- Not restructuring the crate layout (stays as single crate with `#[path]` includes)
- Not removing or deprecating existing APIs unless they conflict
- Not adding a full metrics dashboard (Prometheus + Grafana are external)

## Decisions

### Decision 1: Feature-gated optional dependencies for all new subsystems

**Rationale:** The single-crate architecture means all deps share root `Cargo.toml`. Adding `opentelemetry-otlp`, `multer`, and other heavy deps unconditionally would bloat binary size and compile times for users who don't need them.

**Approach:**
```toml
[features]
telemetry = ["opentelemetry", "opentelemetry-otlp", "tracing-opentelemetry"]
multipart = ["multer"]
static-files = ["tower-http/fs"]
resilience-ext = ["tokio/time"]
backtrace = []
```

Subsystem code uses `#[cfg(feature = "telemetry")]` guards. The `init_tracing` function checks the config at runtime but only does OTLP setup when the feature is enabled.

### Decision 2: Lock-free histogram metrics at record time

**Rationale:** The current `Mutex<MetricsStore>` with `Vec<f64>` storing every individual latency is not scalable. Under load, the Vec grows unboundedly and sorting on scrape blocks all request recording.

**Approach:**
- Replace `Vec<f64>` with a fixed-size `[AtomicU64; 13]` for histogram buckets (same 12 boundaries + overflow)
- Use `AtomicU64` for counters (request count, in-flight, per-status-code counts) → no Mutex contention
- Bucket at record time with a simple `for` loop — O(13) per request
- Store the last N raw latencies in a ring buffer for p50/p90/p99 computation (configurable, default 1000)
- `scrape()` reads atomics without locking
- Per-endpoint: use `HashMap<String, PerEndpointMetrics>` behind a single RwLock (read-heavy)

### Decision 3: DI-based composite health checks

**Rationale:** The DI container already knows about all registered providers. A `HealthIndicator` trait registered as a multi-provider lets the `HealthController` automatically discover and aggregate health checks without manual wiring.

**Approach:**
- Define `HealthIndicator` trait with `fn name(&self) -> &'static str` and `fn check(&self) -> HealthResult`
- `HealthModule` collects all `Arc<dyn HealthIndicator>` from the container via DI multi-provider pattern
- `/health` returns `{"status": "ok"|"degraded"|"unhealthy", "checks": {"db": "ok", "redis": "unreachable"}}`
- Existing `IntegrationHealth` implementations are wrapped to implement `HealthIndicator`
- Timeout per check (default 5s) so a stuck check doesn't hang the endpoint

### Decision 4: RateLimitBackend trait for pluggable rate limiting

**Rationale:** The `InMemoryRateLimiter` is fine for single-instance dev but useless in multi-instance production. A trait allows switching backends without changing middleware logic.

**Approach:**
- Extract `RateLimitBackend` trait from current `InMemoryRateLimiter` logic
- `InMemoryRateLimiter` implements `RateLimitBackend` (existing behavior, no change)
- `RedisRateLimiter` implements `RateLimitBackend` using `redis` crate (feature-gated)
- `RateLimitMiddleware` accepts `Arc<dyn RateLimitBackend>` instead of concrete type
- Add `X-RateLimit-Limit` and `X-RateLimit-Reset` headers to middleware output

### Decision 5: Environment profiles for configuration

**Rationale:** Users expect `config.toml` for base + `config.prod.toml` for production overrides, auto-detected by `IRONIC_ENV` / `APP_ENV`.

**Approach:**
- Add `ConfigurationLoader::auto_detect_env()` method that reads `IRONIC_ENV` then `APP_ENV`, defaults to `"development"`
- After loading base files, automatically loads `config.{env}.toml` (if exists) as an overlay
- Example: `IRONIC_ENV=prod` loads `config.toml` then overlays `config.prod.toml`

### Decision 6: Multipart via multer crate

**Rationale:** `multer` is async, streaming, and has built-in size limits per field. It integrates cleanly with Axum's extractor pattern.

**Approach:**
- Create `MultipartForm<T>` extractor where `T: DeserializeOwned` (for form fields) + `Vec<UploadedFile>` for files
- `UploadedFile` has `field_name, file_name, content_type, size, data: Vec<u8>` (buffered in memory; streaming for large files is future work)
- `AxumAdapter::request_body_limit()` already exists — multipart enforces per-field limits via multer config

### Decision 7: Static files via tower-http ServeDir

**Rationale:** `tower-http::ServeDir` is production-tested, supports ETags, Cache-Control, compression, and directory listing. No need to reinvent.

**Approach:**
- `AxumAdapter::static_files(path, dir)` adds a route at `GET /{path}/*` that serves files from `dir`
- Returns 404 for missing files, directory index is optional (off by default)
- ETag generation from file metadata, `Cache-Control` header configurable

### Decision 8: Drain timeout with tokio::time::timeout

**Rationale:** Without a drain timeout, long-running requests can hang the shutdown indefinitely during rolling deployments.

**Approach:**
- Add `drain_timeout: Duration` to `AxumAdapter` (default `Duration::from_secs(30)`)
- In `listen()`, wrap `with_graceful_shutdown` with `tokio::time::timeout(drain_timeout, ...)`
- On timeout, remaining in-flight requests are dropped and process exits
- Log a warning with the count of dropped in-flight requests

### Decision 9: Backtrace capture gated behind feature flag

**Rationale:** Backtrace capture in Rust has runtime cost. It should be opt-in for production debugging.

**Approach:**
- Add `HttpError::with_backtrace()` method that captures `std::backtrace::Backtrace` (Rust 1.65+)
- Feature flag `backtrace` enables capture on `internal()` errors
- Backtrace is serialized in debug error responses but never in production (configurable)

## Risks / Trade-offs

- **[Risk] Feature flag explosion**: Each new subsystem adds a feature flag. Users may struggle with the right combination. **Mitigation**: Add a `full` meta-feature that enables all optional systems.
- **[Risk] OTLP dependency weight**: `opentelemetry-otlp` pulls in gRPC and protobuf deps, increasing compile time. **Mitigation**: Feature-gated; default-off. Document the trade-off.
- **[Risk] Histogram precision loss**: Bucket-based histograms lose precision compared to full Vec storage. **Mitigation**: Use Prometheus-recommended exponential buckets (0.001, 0.005, 0.01, ...) for adequate precision in the common range.
- **[Risk] Backwards compatibility**: Changing `MetricsLayer` constructor signatures could break users holding a reference to the type. **Mitigation**: Keep `MetricsLayer::new(MetricsConfig)` signature. Add `MetricsLayer::builder()` for advanced config.
- **[Risk] Redis rate limiter latency**: Redis round-trips add latency to every request. **Mitigation**: Use local sliding window + periodic Redis sync as an alternative strategy. Default stays in-memory.

## Migration Plan

1. **Phase 0 (P0)** — Implement first: telemetry, metrics, health, drain timeout
   - No breaking changes expected — all additions are additive
   - `scrape()` output format gains histogram buckets (additive, not breaking)
   - Deploy and validate in staging for 1 week
2. **Phase 1 (P1)** — Config profiles, Redis rate limiting, per-endpoint metrics
   - Config profiles are opt-in (no change for existing users, no env var required)
   - Rate limit trait change: any custom middleware wrapping `InMemoryRateLimiter` needs `Arc<dyn RateLimitBackend>` instead
3. **Phase 2 (P2)** — Static files, multipart, sessions, backtraces
   - All additive, no breaking changes
4. **Phase 3 (P3)** — Backpressure, OAuth2 callback, config reload, feature toggles
   - Backpressure middleware is opt-in via `.layer()`

**Rollback**: Each phase is independently deployable. Feature flags keep unready code out of production builds.

## Open Questions

- Should `MetricsConfig::per_endpoint` be `true` by default in production? (Performance trade-off: more labels = more cardinality)
- Should `ConfigLoader::auto_detect_env()` be the default or opt-in? (Existing users may not expect automatic file loading)
- For multipart: streaming (tokio channels) vs buffered (Vec<u8>) for file data? Buffered is simpler but memory-intensive for large files.
- Should `HealthIndicator::check()` return a `Health` enum with `Ok | Degraded { message } | Unhealthy { error }`, or a `Result<HealthStatus, Error>`? Enum is more expressive.
- Config hot reload: use notify crate or inotify/kqueue directly? `notify` is higher-level but heavier.
