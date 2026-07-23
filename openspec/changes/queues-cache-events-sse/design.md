## Context

Ironic has trait abstractions and in-memory implementations for cache (`Cache` trait), queues (`Queue` trait), events (`EventBus`), and SSE (`sse_channel()`). The Redis cache backend is a stub that returns errors. The queue has no Redis backend. The event bus is purely programmatic — no declarative listener registration. SSE exists as a raw utility function without framework-level route integration.

## Goals / Non-Goals

**Goals:**
- Working `RedisQueue` with BRPOP/RPUSH, priorities, retry, TTL, and dead letters
- Working `RedisCache` with GET/SETEX/DEL and prefix eviction via SCAN
- `#[cache_key]` and `#[cache_ttl]` parameter macros for declarative cache configuration
- `#[event_handler]` attribute macro that auto-registers listeners on EventBus
- `#[sse]` route attribute with `SseRoute` handler type, reconnection support (`Last-Event-ID`), and platform adapter integration
- Feature flags: `sse` (new), `cache` + `redis` compose, `queues` + `redis` compose
- All new types exported from prelude when their feature flags are enabled

**Non-Goals:**
- No RabbitMQ or Kafka queue backends (those are separate changes)
- No distributed cache coherence protocols
- No SSE clustering/room broadcast (existing `WsConnections` covers that)
- No persistent event bus storage (events remain in-memory)

## Decisions

### 1. RedisQueue: List-based with priority sub-queues

Use Redis sorted sets for priority and lists for standard messages. A priority message goes to a sorted set keyed by priority score; a non-priority BRPOP worker drains the sorted set first.

- `ironic:queue:{name}:messages` — LIST for normal messages (RPUSH / BRPOP)
- `ironic:queue:{name}:priority` — SORTED SET keyed by priority score
- `ironic:queue:{name}:processing` — SET for in-flight message IDs (for retry tracking)
- `ironic:queue:{name}:dead` — LIST for dead letters after max retries
- Message payload includes `id`, `retry_count`, `max_retries`, `ttl`, `headers`, `payload`
- `dequeue()` returns a message; consumer must `acknowledge(id)` within a visibility timeout or it re-appears
- Alternative considered: Redis Streams (XADD/XREADGROUP) — more feature-rich but higher complexity and requires consumer groups. Lists + sorted sets are simpler, composable, and sufficient for the use case.

### 2. RedisCache: Direct GET/SETEX/DEL with SCAN for prefix

Replace the stub methods with real Redis commands. Use `SETEX` for TTL-bearing sets, `SET` for no-TTL, `GET` for reads, `DEL` for single removes, `SCAN` (not `KEYS`) for prefix-based eviction.

- Key format: `{prefix}:{key}` where prefix defaults to `ironic:cache`
- `set()` with TTL → `SETEX key ttl value`; without TTL → `SET key value`
- `remove_by_prefix()` → `SCAN 0 MATCH {prefix}:{key}* COUNT 100` in a loop
- RedisCache lives in `crates/ironic-services/src/cache.rs` alongside InMemoryCache
- No new crate needed

### 3. `#[cache_key]` and `#[cache_ttl]` parameter macros

Attribute macros on route handler parameters that attach `CacheKeyComponent` / `CacheTtlMetadata` to the function's metadata. The existing `CacheInterceptor` in `src/cache_interceptor.rs` reads these at runtime to compose cache keys and TTL.

- `#[cache_key]` on a parameter marks it as part of the cache key (concatenated into key string)
- `#[cache_ttl]` on a `Duration` or `u64` parameter dynamically overrides the route-level `#[cache(ttl_secs = N)]`
- These are marker attributes (like `#[get]`, `#[post]`) — they consume no proc-macro input, just attach metadata
- Macro implementation mirrors existing patterns in `ironic-macros/src/lib.rs` (see `marker_attribute!`)

### 4. `#[event_handler]` attribute macro

A proc-macro attribute on methods that generates a registration call in the containing module's initialization. The macro:
- Parses the event type from the method signature's parameter
- Generates a function `__event_handler_<fn>()` that calls `event_bus.subscribe::<E>()` and spawns a tokio task
- The module system (Module derive) calls these during provider registration
- Alternative considered: a standalone registration function — less ergonomic, requires manual wiring

### 5. SSE framework integration: `#[sse]` route + `SseRoute`

A new `#[sse]` attribute macro on route handler functions that:
- Wraps the handler in an SSE endpoint
- Handler signature: `async fn handler(..., sse: SseRoute) -> impl IntoResponse` where `SseRoute` provides `send(event)` and handles `Last-Event-ID` reconnection
- `SseRoute` manages a bounded `mpsc::Sender` + reconnection logic: reads `Last-Event-ID` header, sends missed events from an in-memory buffer
- Platform adapter (`ironic-platform-axum`) maps `#[sse]` routes to Axum's `Sse` response type
- Feature flag: `sse = ["axum/json"]` (axum already has SSE types built-in)
- Alternative considered: gating SSE behind `realtime` — separate feature is cleaner for dependency control

## Risks / Trade-offs

- [RedisQueue visibility timeout] → If a consumer crashes without acknowledging, the message is lost until manual recovery. Mitigation: include `heartbeat` mechanism in a future iteration.
- [Redis SCAN vs KEYS for prefix eviction] → `KEYS` blocks on large datasets; `SCAN` is cursor-based but slower. Mitigation: `SCAN` is the correct choice for production; accept the minor perf trade-off.
- [SSE Last-Event-ID buffer size] → Unlimited buffering leaks memory. Mitigation: bounded ring buffer (configurable, default 1024 events per connection).
- [EventHandler macro spawning tasks] → Each listener spawns a tokio task. Mitigation: tasks are lightweight; add `capacity` parameter to control backpressure.
