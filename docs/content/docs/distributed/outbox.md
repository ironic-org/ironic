---
title: Transactional Outbox & Inbox
description: Reliable at-least-once event delivery with idempotent consumption.
---

# Transactional Outbox & Inbox

The outbox pattern closes the **dual-write gap**: when you write business state to a database and then publish an event to a broker, the two writes are not atomic. If the publish fails or the process crashes between them, the event is lost. The outbox stores the event in the **same transaction** as the business change, and a background relay publishes it later. The inbox pattern makes consumers idempotent so at-least-once delivery is handled exactly once.

## Enabling

```toml
[dependencies]
ironic = { version = "1.0", features = ["outbox"] }
```

To route relayed records onto a queue, also enable `queues`:

```toml
ironic = { version = "1.0", features = ["outbox", "queues"] }
```

## How it works

```
Business transaction
  ├── UPDATE orders SET status='paid' WHERE id=...
  └── INSERT outbox (event_type='order.paid', payload=...)   ← same tx
                    │
                    ▼
OutboxRelay (background task)
  ├── claim pending records (lease)
  ├── publish via RelaySink → Queue / transport
  └── mark Published  |  retry with backoff  |  Dead after max attempts
                    │
                    ▼
Consumer → InboxConsumer (dedup by message id)
```

## Producing events atomically

`TransactionalOutbox::enqueue` appends an outbox record as part of your own transaction handle, so it commits or rolls back with the business change.

```rust
use ironic::distributed::outbox::{
    InMemoryOutboxStore, TransactionalOutbox,
};
use std::sync::Arc;

let store = Arc::new(InMemoryOutboxStore::new());
let outbox = TransactionalOutbox::new(store.clone());

// Inside your DB transaction:
let id = outbox
    .enqueue(&(), "order.paid", b"{\"order_id\":123}")
    .await?;
```

The first argument is the store's transaction handle. For the in-memory store it is `()`; durable sqlx/seaorm stores will accept their transaction type instead. The returned `id` is the message id used for downstream deduplication.

> The in-memory store is **not durable**. Use it for tests and development; supply a durable `OutboxStore` implementation for production.

## Relaying records

An `OutboxRelay` polls pending records, publishes each through a `RelaySink`, and marks them published only after the sink accepts them. Failures are retried with exponential backoff and dead-lettered after `max_attempts`.

```rust
use ironic::distributed::outbox::{
    InMemoryOutboxStore, InMemorySink, OutboxRelay, RelayConfig, TransactionalOutbox,
};
use std::sync::Arc;

let store = Arc::new(InMemoryOutboxStore::new());
let outbox = TransactionalOutbox::new(store.clone());

// Default sink: in-memory. For a queue, use QueueSink::new(Arc<dyn Queue>).
let sink = Arc::new(InMemorySink::new());
let config = RelayConfig::default();
let relay = OutboxRelay::new(store, sink, config);

// Poll once (useful in tests):
let handled = relay.poll_once().await?;

// Or run the loop until told to stop:
// let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
// tokio::spawn({ let relay = relay.clone(); async move { relay.run(stop_rx).await } });
```

### `RelayConfig`

| Field | Type | Default | Description |
|---|---|---|---|
| `poll_interval_ms` | `u64` | `1000` | Idle poll interval |
| `batch_size` | `usize` | `32` | Max records claimed per poll |
| `max_attempts` | `u32` | `3` | Delivery attempts before dead-letter |
| `lease_secs` | `u64` | `60` | Claim lease before a record is re-claimable |
| `backoff_base_ms` | `u64` | `200` | Base backoff for repeated poll failures |
| `backoff_max_ms` | `u64` | `5000` | Backoff cap |

## DI wiring

Register the outbox and relay as providers; the relay starts on bootstrap and stops on shutdown. Both provide their own `provider_definition()`; resolve them as the concrete in-memory-backed types.

```rust
use ironic::distributed::outbox::{
    InMemoryOutboxStore, OutboxRelay, RelayConfig, TransactionalOutbox,
};
use ironic::ContainerBuilder;

let mut builder = ContainerBuilder::new();
builder
    .register(TransactionalOutbox::provider_definition())
    .unwrap()
    .register(OutboxRelay::provider_definition())
    .unwrap()
    .register(ProviderDefinition::value(Arc::new(RelayConfig {
        max_attempts: 7,
        ..Default::default()
    })))
    .unwrap();

let container = builder.build();
container.resolve_forward_refs().await.unwrap();

let outbox: Arc<TransactionalOutbox<InMemoryOutboxStore>> =
    container.resolve::<TransactionalOutbox<InMemoryOutboxStore>>().await.unwrap();
let relay: Arc<OutboxRelay<InMemoryOutboxStore>> =
    container.resolve::<OutboxRelay<InMemoryOutboxStore>>().await.unwrap();
```

Inject `TransactionalOutbox` to enqueue events:

```rust
#[derive(Injectable)]
pub struct OrderService {
    outbox: Arc<TransactionalOutbox<InMemoryOutboxStore>>,
}

impl OrderService {
    pub async fn mark_paid(&self) -> Result<(), OutboxError> {
        // ...within your DB transaction...
        self.outbox.enqueue(&(), "order.paid", b"payload").await?;
        Ok(())
    }
}
```

Registering a custom `Arc<dyn RelaySink>` or `RelayConfig` provider overrides the defaults the relay resolves.

## Consuming idempotently (inbox)

Brokers deliver at-least-once, so the relay may deliver a record more than once. Wrap your handler in an `InboxConsumer` to deduplicate by message id — at-most-once *handling* of at-least-once *delivery*.

```rust
use ironic::distributed::inbox::{InMemoryProcessedStore, InboxConsumer};
use std::sync::Arc;

let store = Arc::new(InMemoryProcessedStore::new());
let consumer = InboxConsumer::new(store.clone());

let handled = consumer
    .handle(&message_id, || async {
        // Idempotent business logic
        process_order_paid().await?;
        Ok(())
    })
    .await?;

if handled {
    // First delivery — processed.
} else {
    // Duplicate — skipped.
}
```

Like the outbox, the in-memory processed store is for tests/development; implement `ProcessedStore` against your database for production.

## Marker attributes

`#[outbox]` and `#[inbox]` are marker attributes that annotate handlers and structs so framework tooling can discover them by attribute. They are no-ops at runtime — use them to tag an outbox relay sink or an inbox consumer handler for codegen or introspection.

```rust
use ironic::{inbox, outbox};

#[outbox]
pub struct OrderCreatedSink;

#[inbox]
pub struct OrderPaidConsumer;
```

## Feature flags

| Feature | What it enables |
|---|---|
| `outbox` | Outbox, relay, inbox, and in-memory stores/sinks |
| `outbox` + `queues` | `QueueSink` for routing relayed records onto a `Queue` |
