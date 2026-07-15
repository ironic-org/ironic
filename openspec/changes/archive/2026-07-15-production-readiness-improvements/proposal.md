## Why

Ironic has strong architectural foundations (DI container, module system, pipeline, Tower-based middleware) but critical gaps prevent it from being used in production: OTLP telemetry is a stub, metrics use a global Mutex with unbounded Vec growth, the /health endpoint is static, rate limiting is per-instance only, and there's no support for static files, multipart uploads, config profiles, or session persistence. These gaps will cause incidents in live deployments and frustrate developers migrating from production-grade frameworks.

## What Changes

### Phase 0 — Critical (P0)
- **Telemetry: stub→real** — Wire opentelemetry dependencies, OTLP exporter, W3C trace context propagation, semantic convention attributes. `TelemetryConfig.otlp_endpoint`, `sample_rate`, and `propagate_context` become operational instead of ignored.
- **Metrics: Mutex→HDR histogram** — Replace global `Mutex<MetricsStore>` with lock-free histogram bucketing at record time. Wire `per_endpoint` labels. Expose public `Counter`, `Gauge`, `Histogram` API for user code.
- **Health: static→composite** — Wire `IntegrationHealth` implementations (SQLx, SeaORM, Diesel, Mongo, Redis) into `HealthModule` via DI. Return per-dependency status with aggregate health.
- **Shutdown: add drain timeout** — Configurable `drain_timeout` on `AxumAdapter`. In-flight requests are given the timeout to complete before forced shutdown.

### Phase 1 — Essential (P1)
- **Rate limiting: Redis backend** — Implement `RedisRateLimiter` behind a `RateLimitBackend` trait. Add `X-RateLimit-Limit` and `X-RateLimit-Reset` headers.
- **Config: environment profiles** — Auto-detect `IRONIC_ENV` / `APP_ENV`, load `config.{env}.toml` on top of base `config.toml`.
- **Metrics: per-endpoint wired** — `per_endpoint` config actually emits `{method="...",path="..."}` labels in Prometheus output.

### Phase 2 — Quality (P2)
- **Static file serving** — Integrate `tower-http::ServeDir` via `AxumAdapter::static_files(path)` with ETag and Cache-Control support.
- **Multipart upload** — Multipart form data extractor with streaming per-field, per-file size limits, and configurable total body limits.
- **Session store: Redis backend** — `RedisSessionStore` implementation. Sessions survive restarts.
- **Error backtraces** — Optional backtrace capture in `HttpError` using `#[track_caller]` / `Backtrace` crate, gated behind a feature flag.

### Phase 3 — Future (P3)
- **Backpressure / bulkhead** — `ConcurrencyLimitLayer` with per-route or global max in-flight. Integrate with circuit breaker.
- **OAuth2 callback handler** — Framework-provided `/auth/callback` handler that exchanges authorization code for tokens.
- **Config hot reload** — File watcher on config files triggers re-`load()` without restart.
- **Runtime feature toggles** — Feature flag provider backed by config file or env vars, hot-reloadable.

## Capabilities

### New Capabilities
- `otlp-telemetry`: OpenTelemetry OTLP export with W3C trace context, semantic conventions, and configurable sampling
- `metrics-rewrite`: Lock-free histogram metrics, per-endpoint labels, public Counter/Gauge/Histogram API for user code
- `composite-health`: HealthIndicator trait, DI-based aggregation, per-dependency status reporting
- `config-environments`: Environment-aware config profiles (dev/staging/prod), config hot reload, runtime feature toggles
- `multipart-upload`: Multipart form data extractor with streaming, per-file limits, configurable total body size
- `static-files`: Built-in static file serving with ETag, Cache-Control, and directory index
- `session-persistence`: Redis-backed session store, configurable TTL, serialization
- `resilience-extensions`: Drain timeout on shutdown, backpressure/bulkhead limiter, standard rate limit headers
- `developer-experience`: Error backtraces in HttpError, CLI `lint`/`debug` commands, test fixture/factory utilities

### Modified Capabilities
- `observability`: Expand requirements to cover OTLP export, composite health aggregation, structured logging API, and semantic convention attributes
- `security-middleware`: Enforce Redis rate limiter backend requirement (already in spec but unimplemented); add standard rate limit headers requirement

## Impact

- **New dependencies**: `opentelemetry`, `opentelemetry-otlp`, `opentelemetry_sdk`, `tracing-opentelemetry` (optional, behind `telemetry` feature)
- **New dependencies**: `tower-http` features `fs`, `limit` (optional)
- **New dependencies**: `multer` or `axum-extra` for multipart (optional)
- **Config struct changes**: `TelemetryConfig` behavior changes (otlp_endpoint stops being a no-op, sample_rate takes effect)
- **Breaking changes**: `MetricsLayer` / `MetricsService` types may change if using custom metrics (low risk — mostly additive). `scrape()` output gains histogram + per-endpoint labels.
- **Documentation**: Update `docs/content/docs/observability/` (4 pages), add pages for config profiles, multipart, static files, resilience. Update CLI docs. Add migration guide for breaking changes.
