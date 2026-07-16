---
title: v0.4.x
description: Complete changelog and release notes for the Ironic v0.4.x stable series.
---

## v0.4.7 — July 16, 2026

# v0.4.7

### Fixed
- enhance release script and project generator for better version handling and documentation sync (a8e859e)\n

---
## v0.4.6 — July 16, 2026

# v0.4.6

### Added
- update version to 0.4.6 and enhance OpenAPI support with new attributes (f088ce6)\n
### Fixed
- comment out database module by default with setup guide (a0612d4)\n

---
## v0.4.5 — July 16, 2026

# v0.4.5

---
## v0.4.4 — July 16, 2026

# v0.4.4

### Added
- enhance update command to automatically upgrade to the latest version (24228b6)\n

---
## v0.4.3 — July 16, 2026

# v0.4.3

### Fixed
- update default server host to 0.0.0.0 in multiple examples (435807c)\n- update latest version in BlogIndex to v0.4.2 (2ca67ef)\n

---
## v0.4.2 — July 16, 2026

# v0.4.2

### Fixed
- enable hot-reload feature in Cargo.toml (a87a424)\n- remove redundant command for cleaning stale test cache artifacts (e560244)\n- update release script to check if version is published on crates.io before proceeding (d188dfc)\n
### Changed
- enhance getting started guide with project structure details (eb6ebeb)\n

---
## v0.4.1 — July 15, 2026

# v0.4.1

### Added
- add repository generation support in CLI and refactor todo app (09f74f4)\n- Add comprehensive documentation for Todo API, database migrations, schema, architecture, deployment, and development setup (5034e24)\n- initialize todo application with Ironic framework (4b19726)\n- Enhance database integration documentation with setup instructions and examples (afea150)\n- Add S3 upload documentation and update meta.json to include new page (630047e)\n- Add configuration and migrations metadata, update advanced pages (16d2473)\n- Update blog post for v0.4.0 with production readiness and enterprise features (b5790de)\n- Update release notes for v0.4.0 with detailed features and improvements (336c954)\n- Refactor imports in error and lib modules for better organization (199bc4f)\n
### Fixed
- Update configuration file names in tests for consistency (cc98918)\n- Ensure stale cache artifacts are cleaned on non-Windows runners (4840653)\n- Update actions/checkout version to v5 in CI workflow (e4c9e5d)\n- Clean stale cache artifacts in CI workflow (56a9b2c)\n- Remove redundant import and reorganize imports for clarity (1a4349d)\n
### Changed
- streamline code structure and improve readability across multiple files (3b7b0a2)\n

---
## v0.4.0 — July 15, 2026

# v0.4.0 — Production Readiness & Enterprise Features

v0.4.0 is a major release focused on production-readiness. It brings multipart file uploads, Redis-backed session persistence, OAuth2 callback handling, backpressure/bulkhead protection, error backtraces, configuration hot-reload, feature toggles, and comprehensive documentation.

---

## Multipart Upload

Ironic now has first-class multipart/form-data support via the `MultipartForm<T>` extractor. This combines structured form fields (via `DeserializeOwned`) with file uploads.

```rust
#[derive(serde::Deserialize)]
struct UploadDto {
    title: String,
    description: String,
}

#[post("/upload")]
async fn upload(
    &self,
    #[custom(MultipartForm<UploadDto>)]
    form: MultipartFormData<UploadDto>,
) -> Result<Json<UploadResponse>, HttpError> {
    let file = &form.files[0];
    // file.field_name, file.file_name, file.content_type, file.data
}
```

Configuration includes per-file size limits (default 5 MiB), per-field size limits (default 256 KiB), and total body limits. Exceeding limits returns 413 Payload Too Large. Gated behind the `multipart` feature flag.

---

## Redis Session Persistence

The new `RedisSessionStore` implements the `SessionStore` trait against Redis, serializing sessions as JSON under `ironic:session:{id}` keys with configurable TTL (default 24h).

```rust
let store = RedisSessionStore::new(connection_manager)
    .with_ttl(Duration::from_secs(3600));

// Or via config struct:
let store = RedisSessionStore::with_config(conn, RedisSessionConfig { session_ttl: 7200 });
```

Gated behind `redis` + `sessions` features.

---

## Error Backtraces

`HttpError` now supports capturing Rust backtraces automatically when the `backtrace` feature is enabled. `HttpError::internal()` captures a `Backtrace` at the call site, serialized in debug builds only.

Gated behind the `backtrace` feature flag.

---

## Backpressure / Bulkhead

`ConcurrencyLimitLayer` and `ConcurrencyLimitService` provide bulkhead protection with a configurable concurrency limit. When exceeded, the service returns 503 Service Unavailable.

```rust
AxumAdapter::new(router)
    .max_concurrent_requests(128)
```

The `ConcurrencyLimitService` is infallible (`Error = Infallible`) for `Router::layer()` compatibility. Gated behind `resilience-ext`.

---

## OAuth2 Callback Handler

The OAuth2 module now includes token exchange (`exchange_code()`), state validation (`validate_state()`), and session token storage (`store_tokens_in_session()`). The `ProviderTokenResponse` type provides structured access to access/refresh tokens and expiry.

The exchange is generic over `AsyncHttpClient` — no `reqwest` dependency required. Gated behind `oauth`.

---

## Config Hot Reload & Feature Toggles

`ConfigurationLoader::watch()` spawns a file watcher that reloads configuration on change, communicating updates through a `tokio::sync::watch` channel.

```rust
let watcher: ConfigWatcher<AppConfig> = ConfigurationLoader::new()
    .file("config.toml")
    .watch()
    .await?;
```

`FeatureToggle` allows runtime feature flag control from config:

```rust
let toggle = FeatureToggle::from_root_config("new_checkout");
if toggle.is_enabled() { /* new path */ }
```

Gated behind `hot-reload` (uses `notify` 8.2.0).

---

## Documentation

15+ documentation pages were created or updated, including:

- **Configuration profiles** — environment-aware config with `IRONIC_ENV`, file precedence
- **Multipart uploads** — usage, size limits, error handling
- **Static file serving** — ETag, Cache-Control, directory index
- **Session persistence** — Redis & in-memory stores, TTL configuration
- **OTLP telemetry** — OpenTelemetry export, W3C trace context, sampling
- **Metrics** — per-endpoint labels, histogram buckets, custom metrics API
- **Health checks** — `HealthIndicator` trait, composite endpoint, per-dependency status
- **Middleware** — rate limit backends, header configuration
- **Deployment** — drain timeout, graceful shutdown, rolling deployments

All docs include runnable examples, configuration tables, testing sections, and common mistake tables.

---

## Migration Guide

A comprehensive [migration guide](/docs/migrations/v0.3.x) covers all breaking changes:

- `RateLimitMiddleware` now requires a `RateLimitBackend` trait object
- `MetricsStore` renamed to `MetricsRegistry` with simplified API
- Session middleware now requires explicit store configuration
- Health endpoint returns per-component checks with composite format
- New feature flags: `multipart`, `static-files`, `backtrace`, `resilience-ext`

## Full Changelog

See the [CHANGELOG](https://github.com/ironic-org/ironic/blob/main/CHANGELOG.md) for the complete list of changes.

---
