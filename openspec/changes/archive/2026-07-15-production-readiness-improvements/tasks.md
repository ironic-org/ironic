## 1. Phase 0: OTLP Telemetry (P0)

- [x] 1.1 Add `opentelemetry`, `opentelemetry-otlp`, `opentelemetry_sdk`, `tracing-opentelemetry` as optional deps behind `telemetry` feature in root Cargo.toml
- [x] 1.2 Create `crates/ironic-telemetry/src/otlp.rs` with OTLP exporter wiring — init `opentelemetry_otlp::SpanExporter::builder().with_tonic()` when `otlp_endpoint` is set
- [x] 1.3 Wire `TracingGuard::drop()` to flush and shutdown the OTLP provider (via `OtlpGuard`)
- [x] 1.4 Implement `inject_trace_context()` with W3C `traceparent` header injection (manual construction from span context)
- [x] 1.5 Wire `TelemetryConfig.sample_rate` into the OTLP sampler (`AlwaysOn` / `AlwaysOff` / `TraceIdRatioBased`)
- [x] 1.6 Update `RequestTracing` middleware in `observability.rs` to set semantic convention attributes (`http.method`, `http.url`, `http.status_code`) on the span
- [x] 1.7 Add `#[cfg(feature = "telemetry")]` guards in `ironic-telemetry/src/lib.rs`

## 2. Phase 0: Metrics Rewrite (P0)

- [x] 2.1 Replace `MetricsStore` with histogram buckets using fixed-size `[AtomicU64; 13]` — bucket at record time, not scrape time
- [x] 2.2 Add bounded ring buffer (configurable, default 1000) for raw latency percentile computation
- [x] 2.3 Replace all `Mutex`-guarded counters with `AtomicU64` — request_count, in_flight (status_counts remain behind `Mutex<HashMap>`, dynamic set of codes)
- [x] 2.4 Wire `MetricsConfig.per_endpoint`: record per-endpoint metrics into `HashMap<String, PerEndpointMetrics>` behind `RwLock`
- [x] 2.5 Update `scrape()` to output per-endpoint labels when `per_endpoint` is true
- [x] 2.6 Update `scrape()` to use pre-bucketed histogram instead of sorting `Vec<f64>`
- [x] 2.7 Create public `MetricsRegistry` API with `Counter`, `Gauge`, `Histogram` structs for user-defined metrics
- [x] 2.8 Register `MetricsRegistry` as a DI provider in `MetricsModule`

## 3. Phase 0: Composite Health Checks (P0)

- [x] 3.1 Define `HealthIndicator` trait in `ironic-core/src/health.rs` with `fn name()` and `async fn check()`
- [x] 3.2 Add `HealthStatus` enum: `Ok`, `Degraded { message }`, `Unhealthy { error }`
- [x] 3.3 Add configurable `health_check_timeout` to `HealthModule` (default 5s)
- [x] 3.4 Wrap `IntegrationHealth` implementations (SQLx, SeaORM, Diesel, Mongo, Redis) as `HealthIndicator` providers in their respective modules
- [x] 3.5 Update `HealthController` to collect all `Arc<dyn HealthIndicator>` from DI container and run checks in parallel
- [x] 3.6 Update `/health` response format to `{"status": "ok"|"degraded"|"unhealthy", "checks": {...}}`
- [x] 3.7 Return appropriate HTTP status codes (200/207/503) based on aggregate health

## 4. Phase 0: Drain Timeout (P0)

- [x] 4.1 Add `drain_timeout: Duration` field to `AxumAdapter` (default `Duration::from_secs(30)`)
- [x] 4.2 Add `AxumAdapter::drain_timeout(mut self, timeout: Duration)` builder method
- [x] 4.3 In `listen()`, wrap graceful shutdown with `tokio::time::timeout(drain_timeout, ...)`
- [x] 4.4 Log warning with count of dropped in-flight requests on timeout

## 5. Phase 1: Redis Rate Limiting (P1)

- [x] 5.1 Extract `RateLimitBackend` trait from `InMemoryRateLimiter` — `async fn check_rate_limit(key, max, window) -> RateLimitResult`
- [x] 5.2 Refactor `RateLimitMiddleware` to accept `Arc<dyn RateLimitBackend>` instead of concrete type
- [x] 5.3 Implement `RedisRateLimiter` using `redis` crate — INCR + EXPIRE via atomic pipeline
- [x] 5.4 Add `X-RateLimit-Limit` and `X-RateLimit-Reset` headers to rate limit middleware responses
- [x] 5.5 Gate `RedisRateLimiter` behind `redis` feature flag (already exists in Cargo.toml)

## 6. Phase 1: Configuration Profiles (P1)

- [x] 6.1 Add `ConfigurationLoader::auto_detect_env()` method — reads `IRONIC_ENV` then `APP_ENV`, defaults to `"development"`
- [x] 6.2 Add `ConfigurationLoader::profile(env)` for manual override
- [x] 6.3 After loading base files, auto-load `config.{env}.toml` as overlay (skip silently if not found)
- [x] 6.4 Document profile precedence: env vars > config.{env}.toml > config.toml

## 7. Phase 1: Per-Endpoint Metrics (P1)

- [x] 7.1 Fix `_method` / `_path` unused variables in `MetricsService::call()` — actually record per-endpoint data
- [x] 7.2 Add `PerEndpointMetrics` struct with `AtomicU64` counters per (method, path) pair
- [x] 7.3 Emit `{method="GET",path="/users"}` labels in `scrape()` output when `per_endpoint` is true
- [x] 7.4 Add cardinality warning when per-endpoint metrics exceed 1000 unique label combinations

## 8. Phase 2: Static File Serving (P2)

- [x] 8.1 Add `static_files(route_path, fs_dir)` method to `AxumAdapter`
- [x] 8.2 Wire `tower-http::services::ServeDir` for the configured directory
- [x] 8.3 Add configurable `Cache-Control` header (default `public, max-age=3600`)
- [x] 8.4 Add ETag generation and `If-None-Match` / 304 support (built-in via `ServeDir`)
- [x] 8.5 Gate behind `static-files` feature flag

## 9. Phase 2: Multipart Upload (P2)

- [x] 9.1 Add `multer` dependency with `multipart` feature flag
- [x] 9.2 Create `MultipartForm<T>` extractor with `DeserializeOwned` fields + `Vec<UploadedFile>`
- [x] 9.3 Create `UploadedFile` struct with `field_name`, `file_name`, `content_type`, `size`, `data: Vec<u8>`
- [x] 9.4 Integrate with `AxumAdapter::request_body_limit()` for total body limit
- [x] 9.5 Add per-file `max_file_size` and per-field `max_field_size` configuration
- [x] 9.6 Return 413 Payload Too Large when limits exceeded

## 10. Phase 2: Redis Session Persistence (P2)

- [x] 10.1 Implement `RedisSessionStore` struct implementing `SessionStore` trait
- [x] 10.2 Use `redis` crate commands: `SETEX` for creation, `GET` for retrieval, `DEL` for deletion
- [x] 10.3 Implement JSON serialization for session values (default)
- [x] 10.4 Add configurable `session_ttl` parameter (default 86400s / 24h)
- [x] 10.5 Gate behind `redis` + `sessions` feature flags
- [x] 10.6 Document that `InMemorySessionStore` is for development, `RedisSessionStore` for production

## 11. Phase 2: Error Backtraces (P2)

- [x] 11.1 Add `backtrace` feature flag to root Cargo.toml
- [x] 11.2 Add optional `backtrace: Option<Backtrace>` field to `HttpError`
- [x] 11.3 Add `HttpError::with_backtrace()` / modify `HttpError::internal()` to capture backtrace when feature is enabled
- [x] 11.4 Serialize backtrace in debug error responses, exclude in production

## 12. Phase 3: Backpressure / Bulkhead (P3)

- [x] 12.1 Create `ConcurrencyLimitLayer` and `ConcurrencyLimitService` — track in-flight with `AtomicU64`, reject with 503 when exceeded
- [x] 12.2 Add `AxumAdapter::max_concurrent_requests(n)` builder method
- [x] 12.3 Gate behind `resilience-ext` feature flag
- [x] 12.4 Integrate with circuit breaker: open circuit when concurrency limit is hit repeatedly

## 13. Phase 3: OAuth2 Callback Handler (P3)

- [x] 13.1 Create `OAuth2Callback` controller with POST `/auth/callback` route
- [x] 13.2 Implement token exchange: POST to token endpoint with authorization code, PKCE verifier, client_id, client_secret
- [x] 13.3 Store tokens in session or return to frontend via redirect
- [x] 13.4 Implement state validation (CSRF token match from authorization request)
- [x] 13.5 Gate behind `oauth` feature flag

## 14. Phase 3: Config Hot Reload & Feature Toggles (P3)

- [x] 14.1 Add `ConfigurationLoader::watch()` method — spawn a file watcher task for config files
- [x] 14.2 Implement `on_reload(callback)` that invokes callback with new config on file change
- [x] 14.3 Add `notify` crate dependency behind `hot-reload` feature flag
- [x] 14.4 Create `FeatureToggle` struct that reads boolean flags from config: `features.new_checkout = true`
- [x] 14.5 Support hot-reload for toggles: feature flags update without restart
- [x] 14.6 Document hot-reload limitations (only config values that opt in; DI container changes still require restart)

## 15. Documentation Updates

- [x] 15.1 Update `docs/content/docs/observability/telemetry.md` — add OTLP setup, W3C trace propagation, sampling configuration
- [x] 15.2 Update `docs/content/docs/observability/metrics.md` — add custom metrics API, per-endpoint labels, histogram buckets
- [x] 15.3 Update `docs/content/docs/observability/health-checks.md` — rewrite for composite health, HealthIndicator trait, IntegrationHealth wiring
- [x] 15.4 Create `docs/content/docs/configuration/profiles.md` — environment profiles, file precedence, IRONIC_ENV
- [x] 15.5 Create `docs/content/docs/advanced/multipart.md` — multipart uploads, file size limits, MultipartForm extractor
- [x] 15.6 Create `docs/content/docs/advanced/static-files.md` — static file serving, ETags, Cache-Control
- [x] 15.7 Create `docs/content/docs/advanced/sessions.md` — Redis session store, serialization, TTL configuration
- [x] 15.8 Update `docs/content/docs/http-api/middleware.md` — rate limit backends, HTTP headers, backend selection
- [x] 15.9 Update `docs/content/docs/more/deployment.md` — drain timeout, graceful shutdown, rolling deployments
- [x] 15.10 Update API reference docs for new/changed public types and methods
- [x] 15.11 Add migration guide for breaking changes (rate limit backend trait, metrics API changes)

## 16. Testing & Verification

- [x] 16.1 Unit tests for histogram bucket logic: verify correct bucket assignment for edge values
- [x] 16.2 Unit tests for `RateLimitBackend` trait and `RedisRateLimiter`
- [x] 16.3 Integration tests for composite health endpoint with mock HealthIndicator
- [x] 16.4 Integration tests for static file serving
- [x] 16.5 Integration tests for multipart upload extractor
- [x] 16.6 Integration tests for drain timeout behavior
- [x] 16.7 Integration tests for config profiles with environment variable
- [x] 16.8 Verify existing tests still pass after refactors (metrics, rate limit middleware)
- [x] 16.9 Run `cargo clippy` on all new code
- [x] 16.10 Benchmark metrics overhead before and after rewrite
