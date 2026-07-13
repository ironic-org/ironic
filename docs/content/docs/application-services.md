---
title: Application services
description: Caching with decorators, cron and interval scheduling, typed events, WebSockets, and Server-Sent Events.
---

# Application services

Enable only the services an application uses, or select `application-services` for all of them.

## Cache

`cache` provides an asynchronous `Cache` contract, JSON helpers, TTL handling, and pluggable backends.

```toml
ironic = { features = ["cache"] }
```

| Type | Description |
|------|-------------|
| `InMemoryCache` | Bounded process-local cache with expiry eviction |
| `RedisCache` | Distributed cache backed by Redis (requires `redis` feature) |

Attach cache metadata to routes with `#[cache(ttl_secs = N)]`. The `CacheInterceptor` checks the
cache before invoking the handler and populates it on a cache miss.

```rust
#[cache(ttl_secs = 60)]
#[get("/products")]
async fn list(&self) -> Result<impl IntoFrameworkResponse, HttpError> { ... }
```

See [Cache decorators](/docs/cache-decorators) for full details.

## Scheduling

`scheduling` provides cooperative background tasks with deterministic shutdown.

```toml
ironic = { features = ["scheduling"] }
```

- `interval(Duration)` — fixed-interval tasks with skipped missed ticks.
- `cron_schedule("expr")` — cron-expression scheduling (requires `cron` feature).
- `ScheduledTask::shutdown()` — graceful stop after the current invocation.
- `ScheduledTask::abort()` — immediate termination.

```rust
use ironic::services::scheduling::interval;
use std::time::Duration;

let task = interval(Duration::from_secs(30), || async move {
    reconcile().await;
});
```

Start tasks in `OnModuleInit` and stop them in `OnModuleDestroy`. See [Task scheduling](/docs/scheduling)
for lifecycle integration and cron examples.

## Events

`events` provides a typed, bounded in-process event bus. Publishing applies backpressure.

```toml
ironic = { features = ["events"] }
```

```rust
use ironic::services::events::EventBus;

let bus = EventBus::default();
let mut receiver = bus.subscribe::<String>(16).await;
bus.publish("created".to_string()).await;
assert_eq!(receiver.recv().await.unwrap().as_str(), "created");
```

## Realtime

`realtime` exposes native Axum WebSocket upgrade types and a bounded SSE channel. See
[WebSocket gateways](/docs/websocket-gateways) for gateway classes, message routing, rooms,
and broadcasting.

Background tasks should be stopped from application lifecycle shutdown hooks. In-memory caches and
event buses are process-local and do not coordinate multiple replicas.
