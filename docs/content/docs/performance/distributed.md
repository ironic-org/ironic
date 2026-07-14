---
title: Distributed Systems
description: Build microservices with queues, CQRS, sagas, gRPC, and GraphQL — all integrated with Ironic's DI system.
---

# Distributed Systems

## What you'll learn

- Add message queues for async processing
- Implement CQRS (Command Query Responsibility Segregation)
- Orchestrate distributed transactions with Sagas
- Serve gRPC and GraphQL endpoints

Enable in `Cargo.toml`:

```toml
ironic = { features = ["distributed"] }
# Or pick individual features:
# ironic = { features = ["queues", "cqrs", "grpc"] }
```

---

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

## Sagas

Orchestrate multi-step transactions with compensation:

```rust
#[derive(Saga)]
struct OrderSaga {
    order_id: u64,
    payment_id: Option<u64>,
}

impl Saga for OrderSaga {
    type Input = CreateOrder;
    type Output = u64;

    async fn execute(&mut self, input: Self::Input) -> SagaResult<Self::Output> {
        // Step 1: Reserve inventory
        self.reserve_inventory().await?;
        // Step 2: Process payment
        self.payment_id = Some(self.process_payment().await?);
        // Step 3: Confirm order
        Ok(self.order_id)
    }

    async fn compensate(&mut self) {
        // Rollback: refund payment, release inventory
        if let Some(pid) = self.payment_id {
            self.refund_payment(pid).await;
        }
        self.release_inventory().await;
    }
}
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
- [x] CQRS separates commands (writes) from queries (reads)
- [x] Sagas handle distributed transactions with rollback
- [x] gRPC and GraphQL integrate with Ironic's DI
