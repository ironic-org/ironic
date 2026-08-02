---
title: Distributed Systems
description: Microservices, message queues, events, sagas, the transactional outbox, and more — all integrated with Ironic's DI system.
---

# Distributed Systems

Ironic's distributed toolkit covers async work, inter-service messaging, and
reliable event delivery — all integrated with the DI container and lifecycle
hooks.

## What you'll learn

- Add message queues for async processing
- Build microservices with request-response message handlers
- Publish and consume events across process or service boundaries
- Orchestrate distributed transactions with Sagas
- Guarantee at-least-once event delivery with the transactional outbox

Enable everything in `Cargo.toml`:

```toml
ironic = { features = ["distributed"] }
# Or pick individual features:
# ironic = { features = ["queues", "sagas", "outbox"] }
```

## Section map

| Topic | Page |
|-------|------|
| [Microservices](./microservices) | Transport-based service framework, `#[message_handler]` |
| [Message Handler](./message-handler) | Request-response handler declarations |
| [Transport Backends](./message-transports) | Redis, RabbitMQ, Kafka, NATS, MQTT configuration |
| [Events](./events) | In-process `EventBus` and `#[event_handler]` |
| [Queues](./queues) | At-least-once job queues (`Queue` trait, `RedisQueue`) |
| [Sagas](./sagas) | Multi-step transactions with automatic compensation |
| [Hybrid Application](./hybrid-application) | Run HTTP and microservice servers in one process |
| [Dead Letter Queue](./dead-letter-queue) | Capture undelivered events |
| [Transactional Outbox](./outbox) | Durable event publishing + idempotent consumption |
| CQRS | Command/query separation (below) |
| gRPC | gRPC server via `tonic` (below) |

## Queues

Process work asynchronously:

```rust
use ironic::distributed::queues::InMemoryQueue;

let queue = InMemoryQueue::new();

// Producer
queue.enqueue("send-email", email_payload).await;

// Consumer
let msg = queue.dequeue("send-email").await;
process_email(msg.payload);

// Acknowledge (remove from queue)
queue.ack(msg.id).await;

// Or reject (re-queue for retry)
queue.reject(msg.id).await;
```

Transports available: Redis, RabbitMQ, Kafka.

## CQRS

Separate read and write operations:

```rust
use ironic::distributed::cqrs::{Command, CqrsBus, Query};

// Commands (write)
struct CreateOrder { items: Vec<u64> }
impl Command for CreateOrder { type Result = u64; }

// Queries (read)
struct GetOrder { id: u64 }
impl Query for GetOrder { type Result = Order; }

let bus = CqrsBus::builder()
    .command_handler(|cmd: CreateOrder| async move { Ok(42) })
    .query_handler(|q: GetOrder| async move { Ok(Order { id: q.id, .. }) })
    .build();

let order_id = bus.execute(CreateOrder { items: vec![1, 2] }).await?;
let order = bus.query(GetOrder { id: order_id }).await?;
```

## gRPC

Serve gRPC alongside REST:

```rust
use ironic::distributed::grpc::GrpcService;

let service = GrpcService::new(my_grpc_service);
app.register_service(service);
```

## What you learned

- [x] Queues decouple producers and consumers
- [x] Microservices communicate through message handlers and events
- [x] Sagas handle distributed transactions with rollback
- [x] CQRS separates commands (writes) from queries (reads)
- [x] gRPC integrates with Ironic's DI
- [x] The outbox closes the dual-write gap with at-least-once delivery
