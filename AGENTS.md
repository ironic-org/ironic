## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- After modifying code files in this session, run `python3 -c "from graphify.watch import _rebuild_code; from pathlib Path; _rebuild_code(Path('.'))"` to keep the graph current
- Never push to GitHub. The user handles all git pushing manually.

# Anchored Summary — Queues / Cache / Events / SSE

## Goal
Implement NestJS-inspired production features: Redis-backed queues, Redis cache backend with SCAN-based prefix eviction, declarative `#[cache_key]`/`#[cache_ttl]` parameter decorators, `#[event]` proc-macro (renamed from `#[event_handler]`) with Module auto-registration, and SSE framework integration with broadcast-based `SseRoute`/`SseConfig` plus AxumAdapter mounting.

## Constraints & Preferences
- All new types exported from the prelude when their feature flag is enabled.
- `missing_docs = "deny"` and `clippy::all = "deny"` — all public items need doc comments.
- Feature flags: `sse` (standalone), `queues` + `redis` compose, `cache` + `redis` compose, `events` (standalone).
- `redis 1.3.0` — SCAN via `cmd("SCAN").query_async()` since `scan()`/`scan_match()` return `AsyncIter` in 1.3.0.
- `ironic-macros` uses `::ironic::` paths for generated code (not `core::`).
- `EventBus::subscribe()` is async — generated handler code must `.await`.
- SSE uses `axum::response::sse::Event`/`Sse` (built-in to axum 0.8).
- All features compile independently and together.

## Progress

### Done
- **Redis Queue Backend**: `RedisQueue` with `QueueConfig` (name, prefix, visibility_timeout, max_retries). `enqueue` (RPUSH), `dequeue` (BRPOP), `acknowledge` (SREM), `reject` (dead-letter + retry tracking). Gate: `queues` + `redis`.
- **Redis Cache Backend**: Real `RedisCache` with `GET`/`SETEX`/`DEL`/`SCAN`-based prefix eviction. Gate: `cache` + `redis`.
- **Cache Key/TTL Decorators**: `#[cache_key]`/`#[cache_ttl]` marker attributes in `ironic-macros`. `CacheKeyMetadata`/`CacheTtlMetadata` in `ironic-http`. CacheInterceptor includes full URI (path+query) in cache key.
- **Event Handler Macro**: `#[event]` proc-macro (renamed from `#[event_handler]`; `#[message_handler]` → `#[message]`). Parses event type from single param (supports `Arc<E>`). Generates `__event_reg_<fn>()` registration function. `auto_register` generates `impl AsyncModuleInit` registrar struct. Gate: `events`. 4 integration tests.
- **Transport Provider (EventClient/EventServer)**: `TransportKind`/`TransportConfig` config enum + struct. `TransportClient` dispatch enum (MicroserviceClient is not object-safe). `EventClient`/`EventServer` injectable providers with auto-connect/listen on bootstrap, close on shutdown. `#[event(transport, auto_register)]` resolves `EventServer` from DI container. Gate: `microservices`.
- **SSE Framework Integration**: `SseRoute` (sender), `SseConfig` (reconnect_buffer_size, keep_alive_interval, event_id_prefix), `SseError` (ClientDisconnected). `sse_endpoint()` creates paired sender+stream. `#[sse]` marker attribute.
- **SSE Route Mounting in AxumAdapter**: `AxumAdapter::sse_route(path, tx)` — broadcast-based SSE endpoint. Each GET creates a new stream subscribed to a `tokio::sync::broadcast::Sender`. `EventBroadcaster` type alias exported. Gate: `sse`.
- **Changelog**: 7 entries for RedisQueue, RedisCache, cache_key/cache_ttl, event_handler, SSE, SSE route mounting, transport_provider.

### In Progress / Blocked
- (none)

## Key Decisions
- `RedisQueue` uses Redis lists (RPUSH/BRPOP) for normal messages; priority via sorted sets is stubbed.
- `RedisCache` uses `cmd("SCAN").query_async()` (MATCH+COUNT) instead of `KEYS` (blocking) or `scan_match()` (incompatible type).
- `#[event]` generates sync registration that spawns an async tokio task internally.
- `auto_register` mode generates registrar struct with `impl AsyncModuleInit`.
- SSE endpoint uses `broadcast::Sender<Event>` for multi-client delivery via `futures_util::stream::unfold`.
- SSE routes are registered via `AxumAdapter::sse_route()` builder method (mapped to GET handler in `build()`).
- `#[sse]` marker attribute provides syntax; deeper route compilation (AxumAdapter mapping) deferred.
- `EventClient` uses `TransportClient` private dispatch enum (`MicroserviceClient` has generic methods, not object-safe). `EventServer` uses `Arc<dyn MicroserviceServer>` (object-safe).
- `#[event(transport = "...", auto_register)]` resolves `EventServer` from DI container; handlers register in `AsyncModuleInit` phase before `listen()` in `OnApplicationBootstrap`.

## Changelog Workflow

When making changes during development:
1. Run `./scripts/add-changelog-entry.sh <Category> "Description"` for each meaningful change
   - Categories: `Added`, `Fixed`, `Changed`, `Security` (case-insensitive)
   - Keep entries concise (one line, no trailing period)
2. The entry is appended under the `[Unreleased]` section of `CHANGELOG.md`

At release time:
- `scripts/release.sh` checks if `[Unreleased]` has content
- **If non-empty**: uses that content as the changelog entry (skips git log parsing)
- **If empty**: falls back to parsing `git log --oneline` since last tag (existing behavior)
- The `[Unreleased]` section body is automatically cleared during the insert

Always update `[Unreleased]` entries as you work — the release script will pick them up verbatim.

**Push policy**: After `make release` completes successfully, the user handles pushing to GitHub manually. Do not commit, tag, or push any code.

## Critical Context
- `QueueMessage` has `retry_count`, `max_retries`, `ttl_secs` — all construction sites must include them.
- `RedisCache::remove_by_prefix` uses `cmd("SCAN").arg(cursor).arg("MATCH")...query_async()`.
- `EventBus::subscribe()` returns `EventSubscription<E>` with `.recv().await`.
- SSE `sse_endpoint()` returns `(mpsc::Sender<Result<Event, Infallible>>, Sse<SseStream>)`.
- `AxumAdapter::sse_route()` stores `SseBroadcasterEntry` (path + broadcast::Sender), wired in `build()` as GET route.
- `EventBroadcaster` type alias = `tokio::sync::broadcast::Sender<axum::response::sse::Event>`, exported from prelude.
- All features compile independently and together (2 pre-existing clippy warnings from ironic-macros + ironic-distributed).

## Relevant Files
- `Cargo.toml`: `sse = []` standalone feature
- `crates/ironic-distributed/src/queues.rs` — `Queue` trait, `QueueMessage`, `QueueConfig`, `RedisQueue`
- `crates/ironic-services/src/cache.rs` — `Cache` trait, `InMemoryCache`, `RedisCache`
- `crates/ironic-services/src/sse.rs` — `SseRoute`, `SseConfig`, `SseError`, `sse_endpoint()`
- `crates/ironic-services/src/events.rs` — `EventBus`, `EventSubscription`
- `crates/ironic-distributed/src/transport_provider.rs` — `TransportKind`, `TransportConfig`, `EventClient`, `EventServer`
- `crates/ironic-macros/src/event.rs` — `#[event]` proc-macro (formerly `event_handler.rs`); `crates/ironic-macros/src/message.rs` — `#[message]` proc-macro (formerly `message_handler.rs`)
- `crates/ironic-macros/src/lib.rs` — marker attrs `cache_key`, `cache_ttl`, `sse`
- `crates/ironic-http/src/metadata.rs` — `CacheKeyMetadata`, `CacheTtlMetadata`
- `crates/ironic-platform-axum/src/lib.rs` — `AxumAdapter::sse_route()`, `EventBroadcaster`, SSE route wiring in `build()`
- `src/lib.rs` — feature gates, prelude exports, `EventBroadcaster` in prelude
- `src/cache_interceptor.rs` — cache key includes full URI (path+query)
- `tests/extended_features.rs` — event handler integration tests
- `docs/content/docs/transport/sse.md` — SSE documentation page
