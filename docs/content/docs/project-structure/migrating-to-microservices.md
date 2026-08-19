---
title: Migrating from Monolith to Microservices
description: Evolve a single-service Ironic app into event-driven microservices — without a rewrite.
---

# Migrating from Monolith to Microservices

Your monolith works — it's at 10M users and the scaling pain is real. Before you
commit to a rewrite, know this: **Ironic's single-service layout and its
microservice layout share the same module, DI, and lifecycle model.** The seams
you need already exist. This guide walks a full, worked example of splitting one
domain out of a monolith into its own event-driven service — with reliable
at-least-once delivery, idempotent consumers, and zero transport lock-in.

> **Read first:** this guide assumes you know the building blocks. For the
> reference material see
> [Events (Transport Provider)](/docs/transport/events),
> [Project Structure](/docs/project-structure/overview), and
> [Transactional Outbox](/docs/distributed/outbox).

---

## 1. Should you migrate?

Microservices are a tool, not a badge of scale. They fix **team** problems
(independent deploys, ownership, scaling a hot path in isolation) and create
**operational** problems (distributed failure, eventual consistency, debugging).

| Your situation | Verdict |
|---|---|
| 10M users, one deployable, multiple teams blocked on each other | Migrate |
| 10M users, one small team, but one endpoint is hot | Add queues/outbox first — stay a monolith |
| Independent scaling for one workload (e.g. workers) | Split *just that* path |
| Fast iteration, low latency, strong consistency requirements | Stay a monolith |

If you only need to survive load, the cheaper lever is a
[hybrid application](/docs/distributed/hybrid-application): run the HTTP server
and an in-process `EventServer` in the same binary and use
[queues](/docs/distributed/queues) for async work. This guide shows you the next
step — splitting domains into separate services — and everything here works no
matter how far you go.

---

## 2. Architecture diff

| Aspect | Monolith | Event-driven workspace |
|--------|----------|------------------------|
| Deployable | One binary | One binary per domain |
| DI container | One for all modules | One **per service** |
| Cross-domain calls | Direct function calls | `EventClient::emit` / `#[event]` |
| Consistency | ACID transactions | At-least-once events + outbox/inbox |
| Shared types | Inlined in the crate | `libs/events` workspace crate |
| Scaling | Scale the whole app | Scale a single service |

```
BEFORE (monolith)
┌──────────────────────────────────────────────┐
│ app (one binary, one container)              │
│  ┌──────────┐   direct fn call   ┌─────────┐ │
│  │ Orders    │ ────────────────▶ │ Payments │ │
│  │ Module    │                   │ Module   │ │
│  └──────────┘                    └─────────┘ │
└──────────────────────────────────────────────┘

AFTER (event-driven workspace)
┌─────────────────┐      emit order.created     ┌───────────────────┐
│ orders-service  │ ──────────────────────────▶ │ payment-service   │
│ HTTP + EventCl. │                             │ EventServer       │
└─────────────────┘ ◀────────────────────────── └───────────────────┘
                          payment.completed
```

---

## 3. Prerequisite: run the monolith as a hybrid app

First, prove the transport works *without* splitting anything. Add the features
and mount an `EventServer` beside your existing HTTP server:

```toml
# Cargo.toml
ironic = { version = "1", features = [
    "microservices",   # EventClient / EventServer / TransportConfig
    "events",          # #[event] macro
    "outbox",          # reliable delivery (Step 4)
    "transport-redis", # or transport-kafka / transport-rabbitmq ...
] }
```

```rust
// src/main.rs
use ironic::prelude::*;

#[ironic::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Application::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())          // existing HTTP server stays
        .microservice_server(RedisServer::new( // ... and now also consumes events
            RedisServerConfig::default(),
        ))
        .build()
        .await?
        .listen("0.0.0.0:3000")
        .await?;
    Ok(())
}
```

Env vars:

```bash
KAFKA_BROKERS=redis://127.0.0.1:6379
KAFKA_TOPIC=shop-events
KAFKA_GROUP_ID=app-service
```

Nothing about your controllers, services, or repositories changes. Run it,
deploy it, watch it consume events. The rest of the migration is extraction —
no new infrastructure concepts.

---

## 4. Step 1 — Find the boundaries

Look for **direct cross-domain calls**. That's your seam. Before: `OrdersService`
calls `PaymentsService` directly:

```rust
// BEFORE — monolith: orders module reaches into payments module
#[derive(Injectable)]
pub struct OrdersService {
    payments: Arc<PaymentsService>, // ← direct dependency across domains
}

impl OrdersService {
    pub async fn place_order(&self, customer_email: String, amount: f64) -> Result<(), OrderError> {
        // ...save the order...
        self.payments.charge(customer_email, amount).await?;
        Ok(())
    }
}
```

After: the order boundary emits an event; the payment boundary handles it. Both
sides only know `EventClient` and event types — they no longer import each other.

```rust
// AFTER (still in the monolith) — orders module emits
use ironic::events::EventClient;
use ironic_events::{OrderCreated, PaymentCompleted};

#[derive(Injectable)]
pub struct OrdersService {
    events: Arc<EventClient>,
}

impl OrdersService {
    pub async fn place_order(&self, customer_email: String, amount: f64) -> Result<(), OrderError> {
        // ...save the order in the same DB transaction (Step 4 makes this reliable)...
        self.events
            .emit("order.created", &OrderCreated { customer_email, amount })
            .await?;
        Ok(())
    }
}

// AFTER — payments module consumes; also emits downstream
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated, events: Arc<EventClient>) {
    // process the payment...
    events
        .emit("payment.completed", &PaymentCompleted { /* ... */ })
        .await?;
}
```

Keep `OrdersService` and the payment handler in the same binary for now — you've
only changed *how* the two domains talk. Run the tests. If the event flow works
in-process with `TransportKind::InMemory`, you're ready to split.

---

## 5. Step 2 — Extract the shared events crate

Services can't share inlined types once they're separate binaries. Move the
event structs to a workspace crate both can depend on:

```bash
ironic new my-platform          # or convert manually, see the workspace guide
cd my-platform
ironic generate app orders-service
ironic generate app payment-service
ironic generate library events  # → libs/events
```

```rust
// libs/events/src/lib.rs
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
    pub status: String,
}
```

Both services add `events = { path = "../../libs/events" }` and depend on it. The
event names (`"order.created"`, `"payment.completed"`) are the contract — keep
them stable, since they cross the wire.

---

## 6. Step 3 — Extract one domain at a time

Move a **whole module** into its own service binary. The module's controllers,
services, repositories, and providers move untouched — the module was already a
vertical slice (see [Single Service](/docs/project-structure/single-service)).

For `payment-service`, copy the payments module and wire the transport into its
new `#[module]`:

```rust
// apps/payment-service/src/app.rs
use std::sync::Arc;
use ironic::prelude::*;
use ironic::distributed::transport_provider::{EventClient, EventServer, TransportConfig};

// ── The moved payments module ──
mod payments;

#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer, payments::PaymentsService],
    async_init = [__EventAuto_on_order_created],
    lifecycle_bootstrap = [EventClient, EventServer],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct PaymentServiceModule;

// ── Handler moved with the module ──
#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated, events: Arc<EventClient>) {
    // process the payment, then emit the downstream event
    events.emit("payment.completed", &PaymentCompleted { /* ... */ }).await?;
}
```

> **How the pieces work** — `#[event]` generates an `__EventAuto_*` struct that
> registers the handler with `EventServer` during `async_init`. `EventClient` /
> `EventServer` connect and start listening in `OnApplicationBootstrap` and close
> themselves in `OnApplicationShutdown` — no manual lifecycle code. See the
> [events reference](/docs/transport/events) for the full wiring.

The `orders-service` is a mirror image: it keeps the HTTP controllers, injects
`EventClient` to emit `order.created`, and consumes `payment.completed` with a
`#[event(transport = "payment.completed")]` handler. Add both binaries to the
workspace `Cargo.toml` members and you're deploying two services.

**Extraction order matters.** Split the most independent, lowest-coupling domain
first (notifications, emails, analytics). Each split deletes one direct
dependency from the monolith. Do NOT split everything at once — the monolith
keeps working during the transition, and it's your rollback plan.

---

## 7. Step 4 — Make delivery reliable (outbox + inbox)

Events are now the backbone. If the process crashes between "save order" and
"emit event", or the broker is down, the event is lost. Fix the **dual-write
gap** with the transactional outbox: write the event in the **same transaction**
as the business change, and let a relay publish it later.

Enable the outbox in `orders-service`:

```toml
ironic = { version = "1", features = ["outbox", "queues", "microservices", "events"] }
```

### Produce atomically — inside your DB transaction

```rust
use ironic::distributed::outbox::{OutboxStore, TransactionalOutbox};
use std::sync::Arc;

#[derive(Injectable)]
pub struct OrdersService {
    outbox: Arc<TransactionalOutbox<MyDurableStore>>, // durable store, not InMemory!
}

impl OrdersService {
    pub async fn place_order(&self, tx: &MyTransaction, customer_email: String, amount: f64) -> Result<(), OutboxError> {
        // ...business writes on tx...
        self.outbox
            .enqueue(tx, "order.created", serde_json::to_vec(&OrderCreated {
                customer_email, amount,
            })?)
            .await?;  // commits or rolls back with the business change
        Ok(())
    }
}
```

> `TransactionalOutbox::enqueue(tx, event_type, payload)` takes your DB
> transaction handle as its first argument. The in-memory store uses `()` and is
> **test-only** — implement `OutboxStore` against your database for production.

### Relay to the transport

An `OutboxRelay` polls pending records and publishes each through a `RelaySink`.
For a queue-backed sink:

```rust
use ironic::distributed::outbox::{OutboxRelay, QueueSink, RelayConfig};
use ironic::distributed::queues::{Queue, RedisQueue};

let queue: Arc<dyn Queue> = Arc::new(RedisQueue::new(QueueConfig { /* ... */ }));
let sink = Arc::new(QueueSink::new(queue));

let relay = OutboxRelay::new(store, sink, RelayConfig {
    poll_interval_ms: 1000,
    max_attempts: 5,
    ..Default::default()
});
```

`OutboxRelay` implements `OnApplicationBootstrap` / `OnApplicationShutdown`, so
register it as a provider and it starts polling on startup and drains on
shutdown. Failures retry with exponential backoff and dead-letter after
`max_attempts`.

### Consume idempotently — inbox

Brokers deliver at-least-once, so the relay may deliver an event more than once.
Wrap the consumer side in an `InboxConsumer` to deduplicate by message id —
at-most-once *handling* of at-least-once *delivery*:

```rust
use ironic::distributed::inbox::{InboxConsumer, ProcessedStore};

let consumer = InboxConsumer::new(Arc::new(MyDurableProcessedStore::new()));

#[event(transport = "order.created")]
async fn on_order_created(event: OrderCreated) {
    let handled = consumer.handle(&event.order_id, || async {
        process_payment(&event).await?;  // idempotent business logic
        Ok(())
    }).await?;
    if !handled {
        tracing::debug!("duplicate delivery of order {}", event.order_id);
    }
}
```

> `InMemoryProcessedStore` is for tests/development. Implement `ProcessedStore`
> against your database for production. Together, outbox + inbox give you
> exactly-once handling across a crash, a retry, or a duplicate delivery.

### What about handler errors?

Events that return `Err` are **silently dropped** by the transport — they are
not redelivered automatically. Two options:

1. **Retry within the handler** — the incoming event is *not* redelivered, so
   re-emit or re-queue the work yourself on failure.
2. **Route failures to a dead-letter queue** — persist the failed payload for
   replay. See [Dead Letter Queue](/docs/distributed/dead-letter-queue).

Never let a failed emit escape silently:

```rust
if let Err(e) = events.emit("payment.completed", &PaymentCompleted { /* ... */ }).await {
    tracing::error!(error = %e.0, "failed to emit payment.completed");
    self.dlq.push("payment.completed", payload).await;  // your replay path
}
```

---

## 8. Testing the split

No broker required. `EventServer::paired()` builds an in-memory pair, and
`TransportKind::InMemory` keeps events in-process — so you can test the full
pipeline the same way before and after the split:

```rust
use ironic_events::{OrderCreated, PaymentCompleted};
use ironic::distributed::transport_provider::EventServer;
use std::sync::Arc;

#[tokio::test]
async fn order_payment_pipeline() {
    let (order_client, order_server) = EventServer::paired(16);
    order_server.on_event("payment.completed", /* ... handler ... */);
    order_server.listen().await.unwrap();

    order_client
        .emit("order.created", &OrderCreated { /* ... */ })
        .await
        .unwrap();

    // ...assert downstream effects...
}
```

Per-service integration tests (`cargo test --workspace`) continue to work
unchanged — each service is just a crate with its own container. For the split
services, add a test that boots each one with `TransportKind::InMemory` and
asserts the event flows between them.

---

## 9. Swap transports with zero code changes

The transport is behind `EventClient` / `EventServer`, so switching backends is
a config change, not a code change:

| Aspect | Kafka | Redis |
|---|---|---|
| Cargo feature | `transport-kafka` | `transport-redis` |
| `KAFKA_BROKERS` | `broker:9092` | `redis://redis:6379` |
| Code changes | **none** | **none** |

```toml
# In ALL services, swap the feature:
ironic = { version = "1", features = ["microservices", "events", "transport-redis"] }
```

Your `#[event]` handlers, `emit` calls, outbox relay, and inbox consumers stay
as-is. The [events tutorial](/docs/transport/events-tutorial) demonstrates this
end to end.

---

## 10. Pitfalls & FAQ

**Each service has its own container.** Modules in one service cannot inject
providers from another. Shared code goes in `libs/` workspace crates; anything
cross-service goes over events, gRPC, or HTTP. See
[Monorepo Workspace](/docs/project-structure/workspace).

**`EventServer` needs its own `KAFKA_GROUP_ID`.** Each service must have a
unique consumer group, otherwise events are load-balanced instead of delivered
to every service. Two instances of the *same* service share a group for
competing consumers.

**Events are silently dropped on handler `Err`.** There is no automatic
redelivery. Use the outbox for publishing reliability, the inbox for duplicate
handling, and a DLQ for handler failures.

**In-memory stores are not durable.** `InMemoryOutboxStore`,
`InMemoryProcessedStore`, and `InMemorySink` are for tests and development.
Production needs your own `OutboxStore`, `ProcessedStore`, and a queue/broker
sink.

**Keep event names stable.** `"order.created"` is a wire contract. Rename it in
a coordinated, versioned way across all consumers.

**You don't have to finish.** A hybrid app where HTTP and events coexist in one
binary is a valid end state. Split only the domains that hurt.

---

## Checklist

- [ ] Run the monolith as a hybrid app (`microservice_server` + existing HTTP)
- [ ] Replace one direct cross-domain call with `emit` + `#[event]` — still in-process
- [ ] Extract event types to `libs/events`
- [ ] Split one low-coupling domain into `apps/<domain>`
- [ ] Wire `TransportConfig` / `EventClient` / `EventServer` + `__EventAuto_*` in its module
- [ ] Add the outbox relay and a durable `OutboxStore`
- [ ] Wrap consumers in `InboxConsumer` with a durable `ProcessedStore`
- [ ] Add a DLQ/replay path for handler failures
- [ ] Test the pipeline with `EventServer::paired()` / `TransportKind::InMemory`
- [ ] Give each service a unique `KAFKA_GROUP_ID`
- [ ] Run `cargo test --workspace`
