## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- After modifying code files in this session, run `python3 -c "from graphify.watch import _rebuild_code; from pathlib import Path; _rebuild_code(Path('.'))"` to keep the graph current

# Anchored Summary — Ironic Production-Readiness Improvements

## Goal
Implement production-readiness improvements: multipart upload, Redis session persistence, error backtraces, backpressure/bulkhead, OAuth2 callback handler, config hot-reload, feature toggles, documentation, and testing.

## Constraints & Preferences
- All new types exported from the prelude when their feature flag is enabled.
- `multer 3.1.0` for multipart parsing (already a transitive dep via axum).
- `redis 1.3.0` for Redis session store (already a workspace dep).
- `backtrace` feature uses `std::backtrace::Backtrace` (stable on Rust 1.97).
- `resilience-ext` depends on `resilience` (circuit breaker + retry).
- OAuth2 helpers use `oauth2 5.0.0`; token exchange generic over `AsyncHttpClient`.
- `hot-reload` uses `notify 8.2.0` (made optional).
- All features compile independently and together.

## Progress

### Done
- **17. Time-Series Logging**: Structured JSON logging via `logging` feature. `LogStorage` trait with `FileLogStorage` (writes `.logs/YYYY-MM-DD.jsonl`). `TimeSeriesLayer` captures all `tracing` events. `TimeSeriesModule` + `TimeSeriesConfig`. Pluggable backend for databases. `ironic::log::{info, warn, error, debug, trace}` re-exports from `tracing`. Feature: `logging`. 90 lib tests. New crate: `crates/ironic-logging/`.
- **9. Multipart Upload**: `MultipartForm<T>`, `UploadedFile`, `MultipartConfig` (5 MiB files, 256 KiB fields). 413/400 responses. Gate: `multipart`.
- **10. Redis Session Persistence**: `RedisSessionStore` via `SETEX`/`GET`/`DEL`. JSON under `ironic:session:{id}`. Default 24h TTL. Gate: `redis` + `sessions`.
- **11. Error Backtraces**: `HttpError.backtrace: Option<Arc<Backtrace>>` behind `backtrace`. `internal()` auto-captures; debug-only serialization. Manual `PartialEq`/`Eq`.
- **12. Backpressure / Bulkhead**: `ConcurrencyLimitLayer` + `ConcurrencyLimitService` (AtomicU64). 503 when exceeded. `AxumAdapter::max_concurrent_requests(n)`. Gate: `resilience-ext`.
- **13. OAuth2 Callback Handler**: `exchange_code()`, `validate_state()`, `store_tokens_in_session()`, `ProviderTokenResponse`. Gate: `oauth`.
- **14. Config Hot Reload & Feature Toggles**: `ConfigurationLoader::watch()` → `ConfigWatcher<T>` (tokio watch + notify). `FeatureToggle` with `from_root_config()`, `is_enabled()`, `with_watcher()`. Gate: `hot-reload`.
- **15. Documentation**: Created `profiles.md`, `multipart.md`, `static-files.md`, `sessions.md`. Updated tracing, metrics, health-checks, middleware, deployment docs.
- **16. Testing**: All tests pass (85 lib + 14 integration + 10 UI + 3 extended). `cargo clippy --all-features` passes. Metrics benchmark added — 11 measurements (counter/gauge/histogram ops, concurrent access, scrape, MetricsLayer request latency): 0–1 ns per metric operation; 391 ns raw Axum vs 714 ns with MetricsLayer. CLI dev command gated behind `hot-reload`.

### In Progress / Blocked
- (none)

## Key Decisions
- Multipart parsed from buffered `Vec<u8>` via `futures_util::stream::once` — avoids changing `FrameworkRequest`.
- `RedisSessionStore` serializes JSON manually to avoid touching `Session`/`SessionId` types.
- `ConcurrencyLimitService` is infallible (`Error = Infallible`) for `Router::layer()` compatibility.
- OAuth2 `exchange_code()` is generic over `AsyncHttpClient` — no `reqwest` dependency.
- `watch()` stores sources separately for rebuild; runs on blocking thread; communicates via tokio watch.
- `FeatureToggle` polls live config watcher on each `is_enabled()` call.

## Critical Context
- `ironic-http/src/multipart.rs` uses `multer::Multipart::with_constraints()`. `is_size_limit_error()` matches `FieldSizeExceeded`/`StreamSizeExceeded`.
- `ironic-auth/src/sessions.rs`: `redis::AsyncCommands` behind `#[cfg(all(feature = "redis", feature = "sessions"))]`.
- `ironic-config`: `ConfigWatcher<T>` requires `T: Clone`. `FeatureToggle::from_root_config()` uses `Config::get("features")`. `watch()` panics without file sources.

## Relevant Files
- `Cargo.toml`: features `multipart`, `backtrace`, `resilience-ext`, `hot-reload`, `logging`; `notify` optional
- `crates/ironic-http/src/multipart.rs`
- `crates/ironic-http/src/error.rs`
- `crates/ironic-auth/src/sessions.rs`
- `crates/ironic-auth/src/oauth.rs`
- `crates/ironic-resilience/src/lib.rs`
- `crates/ironic-platform-axum/src/lib.rs`
- `crates/ironic-config/src/lib.rs`: `ConfigWatcher<T>`, `FeatureToggle`
- `crates/ironic-cli/src/commands/dev.rs` + `mod.rs`: dev command gated
- `docs/content/docs/configuration/profiles.md`
- `docs/content/docs/advanced/{multipart,static-files,sessions}.md`
- `docs/content/docs/migrations/v0.3.x.md`
- `crates/ironic/benches/overhead.rs`
- `crates/ironic/benches/metrics.rs`
- `scripts/release.sh`: pushes commit only (no tag); tag created by CI after publish succeeds. Trigger release workflow manually at github.com/ironic-org/ironic/actions/workflows/release.yml with version `vX.Y.Z`
- `crates/ironic-cli/src/generators/project.rs`: `manifest()` uses semver range (`"0.4"`) instead of exact unpublished version; `example_controller()` stripped `#[api]`/`#[resp]`/`#[req_body]` (not in published versions)
