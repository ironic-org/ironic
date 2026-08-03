---
title: Events (Transport Provider)
description: Send and receive events across services using Kafka, Redis, or in-memory transport — no boilerplate.
---

# Events — Cross-Service Communication

Ironic's transport provider lets you send and receive events between microservices without writing boilerplate. It wraps Kafka, Redis pub/sub, or an in-memory channel behind a single `EventClient` / `EventServer` interface.

> **Cross-service vs in-process:** This page covers `EventClient` / `EventServer`
> for sending events between services. For the in-process `EventBus` (local pub/sub
> within one process) and the `#[event]` macro fundamentals, see
> [Events (Distributed)](/docs/distributed/events).

---

> **Full walkthrough:** building three event-driven microservices end to end — see the [Events Tutorial](/docs/transport/events-tutorial).

---

## Connecting to a Transport Backend

Pick your backend, enable its feature, and set the connection config. Everything else is the same.

### Kafka

| Item | Value |
|---|---|
| Cargo feature | `transport-kafka` |
| Env var `KAFKA_BROKERS` | `host:port` (e.g. `127.0.0.1:9092`) |
| Env var `KAFKA_TOPIC` | Topic name (e.g. `shop-events`) |
| Env var `KAFKA_GROUP_ID` | Consumer group (unique per service) |

**docker-compose:**
```yaml
services:
  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181

  kafka:
    image: confluentinc/cp-kafka:latest
    ports: ["9092:9092"]
    depends_on: [zookeeper]
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
```

**Verify connection:**
```bash
# Install kcat (formerly kafkacat)
brew install kcat  # macOS
apt install kcat   # Linux

# Consume messages from the topic
kcat -b 127.0.0.1:9092 -t shop-events -C

# Produce a test message
echo '{"correlation_id":"test","data":"hello"}' | kcat -b 127.0.0.1:9092 -t shop-events -P
```

### Redis

| Item | Value |
|---|---|
| Cargo feature | `transport-redis` |
| Env var `KAFKA_BROKERS` | `redis://host:port` (e.g. `redis://127.0.0.1:6379`) |
| Env var `KAFKA_TOPIC` | Channel name |
| Env var `KAFKA_GROUP_ID` | Not used by Redis |

**docker-compose:**
```yaml
services:
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
```

**Verify connection:**
```bash
# Monitor the Redis channel
redis-cli -h 127.0.0.1 PSUBSCRIBE "*"

# Publish a test message
redis-cli -h 127.0.0.1 PUBLISH shop-events '{"correlation_id":"test","data":"hello"}'
```

### In-Memory (no infrastructure needed)

No external service required. `TransportKind::InMemory` is always available — used for tests and single-process apps. Events don't leave the process.

### Other supported backends

| Backend | Feature | Env var `KAFKA_BROKERS` format |
|---|---|---|
| RabbitMQ | `transport-rabbitmq` | `amqp://guest:guest@127.0.0.1:5672` |
| MQTT | `transport-mqtt` | `mqtt://127.0.0.1:1883` |
| NATS | `transport-nats` | `nats://127.0.0.1:4222` |

### Connection flow (what happens at startup)

```
Service starts
  │
  ▼
DI container creates TransportConfig ─── reads env vars
  │
  ▼
EventClient created ─── wraps the transport client (KafkaProducer, RedisClient, ...)
EventServer created ─── wraps the transport server (KafkaConsumer, RedisPubSub, ...)
  │
  ▼
OnApplicationBootstrap
  ├── EventClient.connect() ─── connects to broker
  └── EventServer.listen()  ─── starts consuming
  │
  ▼
Events flow through the live connection
  │
  ▼
OnApplicationShutdown
  ├── EventClient.close()
  └── EventServer.close()
```


---

## Consume Then Publish — Injecting `EventClient` Into Handlers

A common pattern: receive an event → process → emit a new event. With `#[event]`, just add `events: Arc<EventClient>` as a second parameter:

```rust
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated, events: Arc<EventClient>) {
    // 1. Process the incoming event
    tracing::info!("processing order {}", event.order_id);

    // 2. Emit a new event — EventClient is injected automatically
    let payment = PaymentCompleted { /* ... */ };
    events.emit("payment.completed", &payment).await.unwrap();
}
```

### How it works

The `#[event(transport = "...")]` macro:

1. Generates a registration function that registers the handler with `EventServer`
2. By default, also generates an `AsyncModuleInit` impl that auto-registers during startup
3. Detects additional `Arc<T>` parameters and resolves them from the DI container during registration

**No globals, no workarounds.** The handler is a plain async function with DI-injected dependencies.

### Opt out of auto-register

Auto-register is the default. If you need to control when handlers are registered (e.g., conditional registration based on config, or custom ordering), use `manual_register`:

```rust
#[event(transport = "order.created", manual_register)]
async fn on_order_created(event: OrderCreated) {
    // Handler logic — but no auto-register struct is generated
}
```

When you use `manual_register`, the macro still generates the registration function `__event_reg_on_order_created()`, but it does NOT generate the `__EventAuto_*` struct. You must call it yourself.

#### Step 1 — Add a manual init service

```rust
#[derive(Injectable)]
pub struct HandlerRegistrar {
    server: Arc<EventServer>,
}

impl OnApplicationBootstrap for HandlerRegistrar {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        let server = self.server.clone();
        Box::pin(async move {
            // Register handlers conditionally
            if std::env::var("ENABLE_ORDER_HANDLER").as_deref() == Ok("true") {
                __event_reg_on_order_created(&*server);
                tracing::info!("order.created handler registered");
            }
        })
    }
}
```

#### Step 2 — Register the service in your module

```rust
#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer, HandlerRegistrar],
    lifecycle_bootstrap = [EventClient, EventServer, HandlerRegistrar],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct MyModule;
```

Note: `HandlerRegistrar` is NOT in `async_init` (that's where auto-register structs go). Instead, it registers handlers during `OnApplicationBootstrap`, before the server starts listening.

#### When to use manual_register

| Situation | Use |
|---|---|
| Always register on startup | Default (omit `manual_register`) |
| Conditional registration (feature flag, env var) | `manual_register` + custom init |
| Custom handler ordering | `manual_register` + register in specific order |
| Register from a test without DI | `manual_register` + call directly with `EventServer::paired()` |

```rust
// Test example with manual_register:
#[tokio::test]
async fn test_handler_directly() {
    let (client, server) = EventServer::paired(16);
    __event_reg_on_order_created(&server);
    server.listen().await.unwrap();
    client.emit("order.created", &OrderCreated { ... }).await.unwrap();
}
```

### Multiple injected services

You can inject any service registered in the DI container by adding it as an `Arc<T>` parameter:

```rust
#[event(transport = "order.created")]
async fn on_order_created(
    event: OrderCreated,
    events: Arc<EventClient>,
    db: Arc<DatabaseService>,
    metrics: Arc<MetricsService>,
) {
    db.save_order(&event).await.unwrap();
    metrics.record_order_placed().await;
    events.emit("payment.completed", &PaymentCompleted { /* ... */ }).await.unwrap();
}
```

Each `Arc<T>` parameter is resolved from the container during startup, captured once, and passed to every handler invocation.

### Consuming multiple event types

Add one `#[event]` per event type. Each generates its own registration function:

```rust
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated, events: Arc<EventClient>) {
    // handle order creation
}

#[event(transport = "order.cancelled")]
async fn on_order_cancelled(event: OrderCancelled) {
    // handle cancellation — no emit needed, so no Arc<EventClient>
}

#[event(transport = "payment.completed")]
async fn on_payment_completed(event: PaymentCompleted, events: Arc<EventClient>) {
    // handle payment
}

#[event(transport = "user.deleted")]
async fn on_user_deleted(event: UserDeleted) {
    // handle user deletion
}
```

Each handler can independently choose whether to inject `Arc<EventClient>` — add it only where you need to emit events.

### Producing multiple events from one handler

Call `events.emit()` as many times as you need:

```rust
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated, events: Arc<EventClient>) {
    // Emit multiple downstream events in sequence
    events.emit("inventory.reserve", &ReserveInventory {
        order_id: event.order_id.clone(),
        items: event.items.clone(),
    }).await.unwrap();

    events.emit("analytics.order_placed", &AnalyticsEvent {
        order_id: event.order_id.clone(),
        amount: event.amount,
    }).await.unwrap();

    events.emit("notification.send", &SendEmail {
        recipient: event.customer_email.clone(),
        template: "order_confirmation".into(),
    }).await.unwrap();
}
```

### Registering many handlers in the module

Each `#[event]` generates a `__EventAuto_<fn>` struct (auto-register is the default). List all of them in `async_init`:

```rust
#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer],
    async_init = [
        __EventAuto_on_order_created,
        __EventAuto_on_order_cancelled,
        __EventAuto_on_payment_completed,
        __EventAuto_on_user_deleted,
    ],
    lifecycle_bootstrap = [EventClient, EventServer],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct AppModule;
```

Every handler in `async_init` gets registered with `EventServer` before `listen()` starts.

---

---

## Full configuration reference

### `TransportConfig`

| Field | Type | Description |
|---|---|---|
| `kind` | `TransportKind` | Backend: `Kafka`, `Redis`, or `InMemory` |
| `brokers` | `String` | Connection URL / broker list |
| `topic` | `String` | Topic or channel name |
| `group_id` | `String` | Consumer group (ignored by Redis/InMemory) |

### `EventClient`

| Method | Description |
|---|---|
| `emit(pattern, event)` | Fire-and-forget event |

### `EventServer`

| Method | Description |
|---|---|
| `on_event(pattern, handler)` | Register handler (called by `#[event]`) |
| `on_message(pattern, handler)` | Register request-response handler |
| `listen()` | Start consuming (called by lifecycle) |
| `close()` | Stop consuming (called by lifecycle shutdown) |

---

---

## Feature flags

| Feature | What it enables |
|---|---|
| `microservices` | Transport provider (`EventClient`, `EventServer`, `TransportConfig`) |
| `events` | `#[event]` proc macro |
| `transport-kafka` | Kafka backend |
| `transport-redis` | Redis pub/sub backend |
| `transport-rabbitmq` | RabbitMQ backend |
| `transport-mqtt` | MQTT backend |
| `transport-nats` | NATS backend |

You need `microservices` + `events` + at least one transport backend for the full flow.
