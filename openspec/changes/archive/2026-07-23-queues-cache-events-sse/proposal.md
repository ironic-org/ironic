## Why

Ironic's application-services layer (`Cache`, `EventBus`, SSE) and distributed layer (`Queue`) have solid trait abstractions and in-memory implementations, but lack production-grade backends and framework-level ergonomics. This blocks adoption in real-world backend scenarios where Redis-backed queues, cache multi-store, declarative event handlers, and framework-integrated SSE are table stakes.

## What Changes

- **Queue – Redis backend**: Implement `RedisQueue` with `BRPOP`/`RPUSH`, message IDs, retry counts, priority queues, dead-letter support, and TTL-based message expiry. Gate: `queues` + `redis`.
- **Cache – Redis backend**: Complete the stub `RedisCache` with `GET`/`SETEX`/`DEL` operations, prefix-based eviction using `SCAN` (replacing `KEYS`), and TTL passthrough. Gate: `cache` + `redis`.
- **Cache – `@CacheKey`/`@CacheTTL` decorator macros**: Implement `#[cache_key]` and `#[cache_ttl]` parameter-level attribute macros in `ironic-macros` for declarative cache key and TTL configuration on route handlers. Gate: `cache`.
- **Events – `@EventHandler` decorator macro**: Add `#[event_handler]` that registers a method as a listener on the `EventBus`, auto-subscribing at module init. Gate: `events`.
- **SSE – Framework integration**: Add `#[sse]` route attribute macro, `SseRoute` extractor for `ironic-platform-axum`, dedicated SSE handler type with reconnection support (`Last-Event-ID`), and `ironic::sse` module. Gate: `sse` (new feature, depends on `axum`).

## Capabilities

### New Capabilities
- `queue-redis-backend`: Redis-backed queue implementation with priorities, retry, TTL, and dead letters
- `event-handler-decorator`: `#[event_handler]` macro for declarative event listener registration
- `sse-framework-integration`: `#[sse]` route attribute, SSE handler type, reconnection support, platform adapter integration

### Modified Capabilities
- `cache-decorators`: Existing spec requires `@CacheKey`/`@CacheTTL` parameter decorators and Redis cache backend — these remain the same requirements, now getting implemented

## Impact

- **New crate capability**: No new crates — Redis queue goes in `crates/ironic-distributed/src/queues/`, Redis cache improvements in `crates/ironic-services/src/cache.rs`, event handler macro in `crates/ironic-macros/`, SSE integration in `crates/ironic-services/src/realtime.rs` + `crates/ironic-platform-axum/src/lib.rs`.
- **New feature flags**: `sse` (depends on `axum`)
- **Modified feature flags**: `cache` gains optional `redis` dependency; `queues` gains optional `redis` dependency.
- **Dependencies**: `redis` already a workspace dep; no new external crates needed.
- **`ironic-macros`**: New `#[cache_key]`, `#[cache_ttl]`, `#[event_handler]`, `#[sse]` attribute macros.
- **Docs**: New `sse.md` transport doc; update `queues.md`, `cache.md`, `events.md` docs.
- **Prelude**: New types exported when their feature flags are enabled.
