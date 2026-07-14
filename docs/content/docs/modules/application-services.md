---
title: Application Services
description: Caching, scheduling, events, and real-time WebSocket communication — all in one feature bundle.
---

# Application Services

Enable all four services with one feature:

```toml
ironic = { features = ["application-services"] }
```

Or pick individual services:

```toml
ironic = { features = ["cache"] }       # Response caching
ironic = { features = ["scheduling"] }  # Background jobs
ironic = { features = ["events"] }      # In-process event bus
ironic = { features = ["realtime"] }    # WebSockets + SSE
```

---

## Cache

Cache route responses automatically:

```rust
#[get("/products")]
#[cache(ttl_secs = 60)]
async fn list(&self) -> Result<Json<Vec<Product>>, HttpError> {
    // Result is cached for 60 seconds
}
```

See [Caching](./cache-decorators) for details.

## Scheduling

Run background tasks:

```rust
use ironic::services::scheduling::interval;
use std::time::Duration;

interval(Duration::from_secs(30), || async move {
    cleanup_old_data().await;
});
```

```rust
use ironic::services::scheduling::cron_schedule;

cron_schedule("0 3 * * *", || async move {
    generate_daily_report().await;
});
```

See [Task Scheduling](./scheduling) for details.

## Events

Typed in-process pub/sub:

```rust
use ironic::services::events::EventBus;

let bus = EventBus::default();

// Subscribe
let mut receiver = bus.subscribe::<String>(16).await;

// Publish
bus.publish("user.created".to_string()).await;

// Receive
assert_eq!(receiver.recv().await.unwrap(), "user.created");
```

## Realtime

WebSocket gateways:

```rust
#[web_socket_gateway("/chat")]
struct ChatGateway;

#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("Echo: {payload}"))
    }
}
```

See [WebSocket Gateways](./websocket-gateways) for details.

## What you learned

- [x] `application-services` bundles cache, scheduling, events, and realtime
- [x] Each service can be enabled independently
- [x] All services integrate with Ironic's DI and lifecycle
