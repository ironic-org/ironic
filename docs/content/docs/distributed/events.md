---
title: Events
description: In-process and cross-process event handling with EventBus
---

# Events

Ironic provides a typed in-process event bus with optional cross-process
transport support.

> **In-process vs cross-process:** This page covers the in-process `EventBus`
> and the `#[event_handler]` macro. For cross-service events over Kafka, Redis,
> or in-memory transports (`EventClient` / `EventServer`), see
> [Events (Transport Provider)](/docs/transport/events).

## In-Process Events

```rust
use ironic::services::events::EventBus;

let bus = EventBus::default();

let mut sub = bus.subscribe::<String>(16).await;
tokio::spawn(async move {
    while let Some(event) = sub.recv().await {
        println!("received: {event}");
    }
});

bus.publish("hello".to_string()).await;
```

## Event Handler Macro

```rust
use ironic::event_handler;

#[event_handler(capacity = 64)]
async fn handle_order_placed(event: Arc<OrderPlaced>) {
    tracing::info!("order placed: {}", event.order_id);
}
```

## Cross-Process Events

Use the `transport` parameter to route events through a microservice transport:

```rust
#[event_handler(transport = "user.created")]
async fn handle_user_created(event: Arc<UserCreated>) {
    tracing::info!("user created in another service: {}", event.user_id);
}
```

This registers the handler on the application's `MicroserviceServer`. Events
published from other services via `client.emit("user.created", event)` will
be delivered to this handler.

## Dead-Letter Queue

Undelivered events are captured:

```rust
let dead = bus.drain_dead_letters().await;
for event in dead {
    tracing::warn!("undelivered event: {event:?}");
}
```
