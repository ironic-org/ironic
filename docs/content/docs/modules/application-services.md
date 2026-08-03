---
title: Application Services
description: Caching, scheduling, events, SSE, and WebSocket communication — all integrated with DI and lifecycle
---

# Application Services

Ironic provides five application services as optional feature-gated modules.
Enable them all at once, or pick individual ones:

```toml
# Enable everything
ironic = { features = ["application-services"] }

# Or pick individually:
ironic = { features = ["cache"] }       # Response caching + RedisCache
ironic = { features = ["scheduling"] }  # Interval + cron background jobs
ironic = { features = ["events"] }      # Typed in-process event bus
ironic = { features = ["sse"] }         # Server-Sent Events
ironic = { features = ["realtime"] }    # WebSocket gateways
```

---

## 1. Cache Service

Two backends: in-memory (default) and Redis.

### Route-Level Caching

```rust
use ironic::prelude::*;

#[get("/products")]
#[cache(ttl_secs = 60)]
async fn list_products(&self) -> Json<Vec<Product>> {
    // Response is cached for 60 seconds
    // Subsequent requests within TTL return cached response
    self.service.list().await.into()
}
```

### Programmatic Caching

```rust
use ironic::services::cache::{Cache, InMemoryCache};

let cache = InMemoryCache::new(100);  // max 100 entries
cache.set("key", b"value", Some(Duration::from_secs(30))).await;
let value = cache.get("key").await;
cache.remove("key").await;

// JSON convenience
cache.set_json("user:1", &user, None).await;
let user: Option<User> = cache.get_json("user:1").await;
```

### Redis Cache

```rust
use ironic::services::cache::RedisCache;
use redis::aio::ConnectionManager;

let connection = ConnectionManager::new(client).await?;
let cache = RedisCache::new(connection);
cache.set_json("key", &value, Some(Duration::from_secs(60))).await;

// SCAN-based prefix invalidation
cache.remove_by_prefix("user:").await;  // deletes all user:* keys
```

### Custom Cache Key

```rust
#[get("/products/{id}")]
#[cache(ttl_secs = 60)]
async fn get_product(
    #[param("id")] id: u64,
    #[cache_key] currency: String,  // included in cache key
) -> Json<Product> {
    // Cache key = "/products/42?currency=USD"
    self.service.find(id, &currency).await.into()
}
```

### Dynamic TTL

```rust
#[get("/products/{id}")]
#[cache]
async fn get_product(
    #[param("id")] id: u64,
    #[cache_ttl] ttl: Duration,  // overrides route-level TTL per-request
) -> Json<Product> {
    // Cache for `ttl` duration specified per-request
    self.service.find(id).await.into()
}
```

### Cache Invalidation

```rust
use ironic::services::cache::CacheInterceptor;

// CacheInterceptor is automatically registered when cache feature is enabled
// It caches GET responses based on #[cache] attributes
// POST/PUT/DELETE requests invalidate related cache entries
```

**Reference:** [Cache Decorators](/docs/performance/cache-decorators)

---

## 2. Scheduling Service

Run background tasks on an interval or cron schedule.

### Fixed Interval

```rust
use ironic::services::scheduling::interval;
use std::time::Duration;

// Run every 30 seconds
let task = interval(Duration::from_secs(30), || async move {
    cleanup_expired_sessions().await;
}).await;

// Control the task at runtime
task.pause().await;
task.resume().await;
task.abort().await;
```

### Cron Schedule

```rust
use ironic::services::scheduling::cron_schedule;

// Run daily at 3:00 AM
let task = cron_schedule("0 3 * * *", || async move {
    generate_daily_report().await;
}).await;
```

### Managed Task

```rust
use ironic::services::scheduling::ScheduledTask;

// ScheduledTask handle provides lifecycle control
pub struct TaskManager {
    tasks: Vec<ScheduledTask>,
}

impl TaskManager {
    pub async fn start_all(&mut self) {
        self.tasks.push(
            interval(Duration::from_secs(60), || async {
                sync_data().await;
            }).await
        );
    }

    pub async fn shutdown(&self) {
        for task in &self.tasks {
            task.shutdown().await;
        }
    }
}
```

**Reference:** [Task Scheduling](/docs/performance/scheduling)

---

## 3. Events Service

Typed in-process pub/sub with optional cross-process transport.

### Basic Pub/Sub

```rust
use ironic::services::events::EventBus;

let bus = EventBus::default();

// Subscribe (capacity = 16 messages buffer)
let mut receiver = bus.subscribe::<OrderPlaced>(16).await;

// Publish
bus.publish(OrderPlaced { id: 123 }).await;

// Receive
while let Some(event) = receiver.recv().await {
    tracing::info!("processing order: {}", event.id);
}
```

### Event Handler Macro

```rust
use ironic::event;
use std::sync::Arc;

#[event(capacity = 64)]
async fn on_order_placed(event: Arc<OrderPlaced>) {
    // Runs in a background tokio task
    process_order(event).await;
}
```

### Cross-Process Events

```rust
#[event(transport = "order.created")]
async fn on_order_created(event: Arc<OrderEvent>) {
    // Receives events published by OTHER services via Redis
    tracing::info!("cross-service event: {}", event.id);
}

// Publishing service:
client.emit("order.created", &event).await?;
```

### Dead-Letter Queue

```rust
// Undelivered events (no subscribers) are captured
let dead = bus.drain_dead_letters().await;
for event in dead {
    tracing::warn!("undelivered event: {event:?}");
}
```

**Reference:** [Events](/docs/distributed/events)

---

## 4. SSE Service (Server-Sent Events)

Push real-time events to connected HTTP clients.

### Basic SSE Endpoint

```rust
use ironic::services::sse::{SseRoute, SseConfig, sse_endpoint};
use axum::response::sse::Event;

let (tx, stream) = sse_endpoint(SseConfig::default());

// Mount the SSE route
Application::builder()
    .platform(
        AxumAdapter::new()
            .sse_route("/events", tx)
    )
    .build()
    .await?;

// Broadcast events to all connected clients
tx.send(Event::default().data("hello")).ok();
```

### Broadcasting

```rust
use ironic::EventBroadcaster;
use tokio::sync::broadcast;

let (tx, _) = broadcast::channel::<Event>(100);
let broadcaster: EventBroadcaster = tx;

// Any service can inject EventBroadcaster
// and push events to all SSE clients
broadcaster.send(Event::default().json(json)).ok();
```

**Reference:** [SSE](/docs/transport/sse)

---

## 5. Realtime Service (WebSockets)

Full-duplex WebSocket communication with rooms and broadcasting.

### WebSocket Gateway

```rust
use ironic::prelude::*;

#[web_socket_gateway("/chat")]
pub struct ChatGateway;

#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("echo: {payload}"))
    }

    #[subscribe_message("join")]
    async fn on_join(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("welcome {payload}"))
    }
}
```

### Connection Management

```rust
use ironic::services::ws::WsConnections;

#[derive(Injectable)]
pub struct ChatService {
    connections: Arc<WsConnections>,
}

impl ChatService {
    pub async fn broadcast(&self, room: &str, message: &str) {
        // Send to all connections in a room
        self.connections.broadcast(room, message).await;
    }
}
```

**Reference:** [WebSocket Gateways](/docs/transport/websocket)

---

## Feature Flag Reference

| Feature | What it enables | Dependencies |
|---------|----------------|--------------|
| `cache` | `Cache` trait, `InMemoryCache`, `RedisCache`, `CacheInterceptor` | — |
| `scheduling` | `ScheduledTask`, `interval()`, `cron()`, `cron_schedule()` | `cron` (optional) |
| `events` | `EventBus`, `EventSubscription`, `#[event]` macro | — |
| `sse` | `SseRoute`, `SseConfig`, `sse_endpoint()`, `EventBroadcaster` | — |
| `realtime` | `WebSocketHandler`, `WsConnections`, `#[web_socket_gateway]` | `axum/ws` |
| `application-services` | All five above | All of the above |

## DI Integration

All services integrate with Ironic's DI container and lifecycle:

```rust
#[derive(Injectable)]
pub struct OrderService {
    cache: Arc<InMemoryCache>,         // from cache feature
    event_bus: Arc<EventBus>,         // from events feature
    connections: Arc<WsConnections>,  // from realtime feature
}
```

## What you learned

- [x] 5 application services: cache, scheduling, events, SSE, WebSocket
- [x] Enable individually or via `application-services` bundle
- [x] Cache with in-memory or Redis backends, route-level `#[cache]` decorator
- [x] Scheduling with `interval()` and `cron()` with pause/resume/abort
- [x] Events with in-process pub/sub and cross-process transport option
- [x] SSE for server-to-client push with broadcast channels
- [x] WebSocket gateways with rooms and broadcasting
- [x] All services injectable via DI container
