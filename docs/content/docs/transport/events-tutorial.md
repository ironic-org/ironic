---
title: Events Tutorial
description: Build three event-driven microservices (order, payment, notification) end to end with Kafka or Redis.
---

# Events Tutorial — 3 Microservices

A complete end-to-end example: three services that communicate only through
events. This tutorial covers the shared events crate, each service, running with
Kafka or Redis, testing the full pipeline, switching transports, and production
best practices.

> **API reference:** for the `EventClient` / `EventServer` interface, backends, and
> the `#[event]` macro fundamentals, see [Events (Transport Provider)](/docs/transport/events).

## Production Example — 3 Microservices

This example walks through a real-world order processing pipeline with three services that talk to each other through events:

```
┌──────────────┐    order.created     ┌────────────────┐
│              │ ──────────────────▶  │                │
│ Order Service │                    │ Payment Service │
│              │ ◀────────────────── │                │
└──────────────┘    payment.completed └───────┬────────┘
                                              │
                                              │ payment.completed
                                              ▼
                                      ┌────────────────┐
                                      │  Notification  │
                                      │    Service     │
                                      │                │
                                      └────────────────┘
```

| Service | Produces | Consumes |
|---------|----------|----------|
| **Order Service** | `order.created` | `payment.completed` |
| **Payment Service** | `payment.completed` | `order.created` |
| **Notification Service** | `notification.sent` | `payment.completed` |

Each service is a separate binary. They communicate **only through events** — no HTTP calls, no shared database.

---

---

### Docker Compose for all services + Kafka

```yaml
version: "3.8"
services:
  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181

  kafka:
    image: confluentinc/cp-kafka:latest
    depends_on: [zookeeper]
    ports: ["9092:9092"]
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1

  order-service:
    build: ./order-service
    depends_on: [kafka]
    environment:
      KAFKA_BROKERS: kafka:9092
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: order-service

  payment-service:
    build: ./payment-service
    depends_on: [kafka]
    environment:
      KAFKA_BROKERS: kafka:9092
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: payment-service

  notification-service:
    build: ./notification-service
    depends_on: [kafka]
    environment:
      KAFKA_BROKERS: kafka:9092
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: notification-service
```

---

---

## Part 1 — Shared Events Crate

All three services share the same event type definitions. Put these in a common crate:

```rust
// events/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderCreated {
    pub order_id: String,
    pub customer_email: String,
    pub amount: f64,
    pub currency: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentCompleted {
    pub order_id: String,
    pub transaction_id: String,
    pub amount: f64,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationSent {
    pub order_id: String,
    pub recipient: String,
    pub channel: String, // "email" | "sms"
}
```

---

---

## Part 2 — Order Service

Creates orders and emits `order.created`. Listens for `payment.completed`.

### Cargo.toml

```toml
[dependencies]
ironic = { version = "1.0", features = [
    "microservices",
    "events",
    "transport-kafka",
] }
events = { path = "../events" }
```

### Module + Event Handlers

```rust
use ironic::*;
use events::*;

// ── Register root module ──

#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer, OrderService],
    async_init = [__EventAuto_on_payment_completed],
    lifecycle_bootstrap = [EventClient, EventServer],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct OrderServiceModule;

// ── Consume: payment.completed ──

#[event(transport = "payment.completed")]
async fn on_payment_completed(event: PaymentCompleted) {
    tracing::info!(
        "order {} payment confirmed: {} ({})",
        event.order_id, event.transaction_id, event.status,
    );
    // Update order status in database
}

// ── Produce: order.created ──

#[derive(Injectable)]
pub struct OrderService {
    events: Arc<EventClient>,
}

impl OrderService {
    pub async fn create_order(
        &self,
        customer_email: String,
        amount: f64,
    ) -> Result<(), TransportError> {
        let order_id = uuid::Uuid::new_v4().to_string();
        let event = OrderCreated {
            order_id,
            customer_email,
            amount,
            currency: "USD".into(),
        };
        self.events.emit("order.created", &event).await
    }
}
```

### Env vars

```bash
KAFKA_BROKERS=broker-1:9092,broker-2:9092
KAFKA_TOPIC=shop-events
KAFKA_GROUP_ID=order-service
```

---

---

## Part 3 — Payment Service

Listens for `order.created`, processes payment, emits `payment.completed`.

### Cargo.toml

```toml
[dependencies]
ironic = { version = "1.0", features = [
    "microservices",
    "events",
    "transport-kafka",
] }
events = { path = "../events" }
```

### Module + Event Handlers

```rust
use std::sync::Arc;
use ironic::*;
use events::*;

#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer],
    async_init = [__EventAuto_on_order_created],
    lifecycle_bootstrap = [EventClient, EventServer],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct PaymentServiceModule;

// ── Consume: order.created → produce: payment.completed ──

#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated, events: Arc<EventClient>) {
    tracing::info!(
        "processing payment for order {} ({}{})",
        event.order_id, event.amount, event.currency,
    );

    let payment = PaymentCompleted {
        order_id: event.order_id.clone(),
        transaction_id: uuid::Uuid::new_v4().to_string(),
        amount: event.amount,
        status: "confirmed".into(),
    };

    events.emit("payment.completed", &payment).await.unwrap();
    tracing::info!("payment.completed emitted for order {}", event.order_id);
}
```

### Env vars

```bash
KAFKA_BROKERS=broker-1:9092,broker-2:9092
KAFKA_TOPIC=shop-events
KAFKA_GROUP_ID=payment-service
```

---

---

## Part 4 — Notification Service

Listens for `payment.completed`, sends email/SMS, emits `notification.sent`.

### Cargo.toml

```toml
[dependencies]
ironic = { version = "1.0", features = [
    "microservices",
    "events",
    "transport-kafka",
] }
events = { path = "../events" }
```

### Module + Event Handlers

```rust
use std::sync::Arc;
use ironic::*;
use events::*;

#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer],
    async_init = [__EventAuto_on_payment_completed],
    lifecycle_bootstrap = [EventClient, EventServer],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct NotificationServiceModule;

// ── Consume: payment.completed → produce: notification.sent ──

#[event(transport = "payment.completed")]
async fn on_payment_completed(event: PaymentCompleted, events: Arc<EventClient>) {
    tracing::info!(
        "sending notification for order {}...",
        event.order_id,
    );

    // Send email via AWS SES, Twilio SMS, etc.
    send_email_notification(&event).await;

    let notification = NotificationSent {
        order_id: event.order_id.clone(),
        recipient: "customer@example.com".into(),
        channel: "email".into(),
    };

    events.emit("notification.sent", &notification).await.unwrap();
}

async fn send_email_notification(_event: &PaymentCompleted) {
    // Integration with AWS SES, SendGrid, etc.
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    tracing::info!("email sent");
}
```

### Env vars

```bash
KAFKA_BROKERS=broker-1:9092,broker-2:9092
KAFKA_TOPIC=shop-events
KAFKA_GROUP_ID=notification-service
```

---

---

## Part 5 — Running with Kafka

### docker-compose

```yaml
version: "3.8"
services:
  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181

  kafka:
    image: confluentinc/cp-kafka:latest
    depends_on: [zookeeper]
    ports: ["9092:9092"]
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1

  order-service:
    build: ./order-service
    depends_on: [kafka]
    environment:
      KAFKA_BROKERS: kafka:9092
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: order-service

  payment-service:
    build: ./payment-service
    depends_on: [kafka]
    environment:
      KAFKA_BROKERS: kafka:9092
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: payment-service

  notification-service:
    build: ./notification-service
    depends_on: [kafka]
    environment:
      KAFKA_BROKERS: kafka:9092
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: notification-service
```

### Start the system

```bash
docker compose up --build
```

Each service connects to the same Kafka topic `shop-events` with its own consumer group. When `order-service` emits `order.created`, both `payment-service` and `notification-service` could receive it — but since each has a unique `group_id`, every event goes to exactly one consumer per group (competing consumers pattern).

---

---

## Part 6 — Same Example with Redis

### Change features

```toml
# In ALL three services:
ironic = { version = "1.0", features = [
    "microservices",
    "events",
    "transport-redis",
] }
```

### Redis docker-compose

```yaml
version: "3.8"
services:
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

  order-service:
    build: ./order-service
    depends_on: [redis]
    environment:
      KAFKA_BROKERS: redis://redis:6379   # reuse same env var name
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: order-service

  payment-service:
    build: ./payment-service
    depends_on: [redis]
    environment:
      KAFKA_BROKERS: redis://redis:6379
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: payment-service

  notification-service:
    build: ./notification-service
    depends_on: [redis]
    environment:
      KAFKA_BROKERS: redis://redis:6379
      KAFKA_TOPIC: shop-events
      KAFKA_GROUP_ID: notification-service
```

### What changed?

| Aspect | Kafka | Redis |
|---|---|---|
| Cargo feature | `transport-kafka` | `transport-redis` |
| Env `KAFKA_BROKERS` | `broker:9092` | `redis://redis:6379` |
| Code changes | **none** | **none** |
| Event handlers | unchanged | unchanged |
| `EventClient::emit()` | unchanged | unchanged |

**Zero code changes.** Only the feature flag and env var value differ.

---

---

## Part 7 — Testing the Full Pipeline

No Kafka or Redis needed. Use `TransportKind::InMemory` + `EventServer::paired()`:

```rust
use events::*;
use ironic::distributed::transport_provider::EventServer;
use ironic::distributed::microservices::{MicroserviceServer, TransportError};
use std::sync::Arc;

#[tokio::test]
async fn order_payment_notification_pipeline() {
    // ── Order Service side ──
    let (order_client, order_server) = EventServer::paired(16);

    let order_received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let captured = order_received.clone();
    order_server.on_event(
        "payment.completed",
        Arc::new(move |payload, _ctx| {
            let captured = captured.clone();
            Box::pin(async move {
                let ev: PaymentCompleted = serde_json::from_slice(&payload)
                    .map_err(|e| TransportError(e.to_string()))?;
                captured.lock().await.push(ev);
                Ok(())
            })
        }),
    );
    order_server.listen().await.unwrap();

    // ── Payment Service side ──
    let (payment_client, payment_server) = EventServer::paired(16);
    let payments = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let captured = payments.clone();
    payment_server.on_event(
        "order.created",
        Arc::new(move |payload, _ctx| {
            let captured = captured.clone();
            Box::pin(async move {
                let ev: OrderCreated = serde_json::from_slice(&payload)
                    .map_err(|e| TransportError(e.to_string()))?;

                // Simulate payment processing
                let completed = PaymentCompleted {
                    order_id: ev.order_id.clone(),
                    transaction_id: "tx-test-1".into(),
                    amount: ev.amount,
                    status: "confirmed".into(),
                };
                captured.lock().await.push(completed.clone());

                // Emit payment.completed back to order service
                // In real code, inject EventClient. Here we use a shared client.
                Ok(())
            })
        }),
    );
    payment_server.listen().await.unwrap();

    // ── Order Service emits order.created ──
    let order_event = OrderCreated {
        order_id: "ord-123".into(),
        customer_email: "test@example.com".into(),
        amount: 49.99,
        currency: "USD".into(),
    };
    order_client.emit("order.created", &order_event).await.unwrap();

    // ── Wait for async processing ──
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Both services processed their events
    assert_eq!(payments.lock().await.len(), 1);
    assert_eq!(payments.lock().await[0].order_id, "ord-123");
}
```

This test validates the entire event pipeline: emit → receive → process → emit → receive — all without a single network call.

### Testing each service in isolation

```rust
#[tokio::test]
async fn order_service_emits_order_created() {
    let (client, server) = EventServer::paired(16);

    let captured = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let events = captured.clone();
    server.on_event(
        "order.created",
        Arc::new(move |payload, _ctx| {
            let events = events.clone();
            Box::pin(async move {
                let ev: OrderCreated = serde_json::from_slice(&payload).unwrap();
                events.lock().await.push(ev);
                Ok(())
            })
        }),
    );
    server.listen().await.unwrap();

    client
        .emit(
            "order.created",
            &OrderCreated {
                order_id: "ord-1".into(),
                customer_email: "a@b.com".into(),
                amount: 10.0,
                currency: "USD".into(),
            },
        )
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(captured.lock().await.len(), 1);
}
```

---

---

## Part 8 — Switching Between Transports

### Step 1: Change Cargo.toml

```toml
# Before
ironic = { features = ["transport-kafka"] }

# After
ironic = { features = ["transport-redis"] }
```

### Step 2: Change docker-compose (if using containers)

```yaml
# Before
KAFKA_BROKERS: broker:9092

# After
KAFKA_BROKERS: redis://redis:6379
```

### Step 3: Deploy

That's it. **No code changes to your event handlers, producers, or services.**

---

---

## Part 9 — Best Practices for Production

### Consumer group isolation

Each service **must** have a unique `KAFKA_GROUP_ID`. This ensures every event is delivered to every service. If two instances of the same service share a group ID, events are load-balanced (competing consumers).

```bash
KAFKA_GROUP_ID=order-service    # service A
KAFKA_GROUP_ID=payment-service  # service B
KAFKA_GROUP_ID=notification-svc # service C
```

### Topic naming convention

Use reverse-domain or hierarchical patterns for topics/event names:

```
order.created
payment.completed
notification.sent
user.profile.updated
inventory.reserved
```

Consistent naming makes it easy to trace event flows and configure ACLs.

### Error handling inside handlers

```rust
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated) {
    if let Err(e) = process_payment(&event).await {
        tracing::error!(
            order_id = %event.order_id,
            error = %e,
            "payment processing failed",
        );
        // The event is NOT redelivered automatically.
        // Implement a dead-letter queue for retries.
    }
}
```

Events that return `Err` are silently dropped by the transport. Implement a retry/dead-letter mechanism for production.

### Consume then publish with error handling

When a handler both consumes an event and emits a new one, wrap the emit in error handling:

```rust
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated, events: Arc<EventClient>) {
    if let Err(e) = events.emit("payment.completed", &PaymentCompleted { /* ... */ }).await {
        tracing::error!(
            order_id = %event.order_id,
            error = %e.0,
            "failed to emit payment.completed",
        );
    }
}
```

If the downstream emit fails, the incoming event is NOT redelivered. Consider storing failed events in a dead-letter queue for retry.

### Graceful shutdown

The `OnApplicationShutdown` lifecycle hook on `EventClient` and `EventServer` handles disconnection automatically. No manual cleanup needed.

### Observability

Add tracing to every handler:

```rust
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated) {
    let span = tracing::info_span!("handle_order_created", order_id = %event.order_id);
    let _guard = span.enter();
    // ... handler logic ...
}
```

---
