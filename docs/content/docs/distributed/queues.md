---
title: Message Queues
description: At-least-once message queue abstractions with a built-in in-memory implementation.
---

# Message Queues

## Enabling

```toml
ironic = { features = ["queues"] }
```

## Queue trait

The `Queue` trait provides an at-least-once delivery contract with four operations:

- **enqueue** — push a message into the queue
- **dequeue** — wait for the next message
- **acknowledge** — mark a message as processed
- **reject** — mark a message as failed, with optional redelivery

```rust
use ironic::distributed::queue::{Queue, QueueMessage, InMemoryQueue};

#[derive(Injectable)]
struct Worker {
    queue: Arc<InMemoryQueue>,
}

impl Worker {
    async fn process(&self) -> Result<(), QueueError> {
        while let Some(msg) = self.queue.dequeue().await? {
            match self.handle(&msg).await {
                Ok(()) => self.queue.acknowledge(&msg.id).await?,
                Err(_) => self.queue.reject(msg, true).await?,  // requeue
            }
        }
        Ok(())
    }

    async fn handle(&self, msg: &QueueMessage) -> Result<(), ()> {
        // process the payload
        Ok(())
    }
}
```

## InMemoryQueue

Bounded process-local queue backed by `tokio::sync::mpsc`. Suitable for tests, development, and single-process workers.

```rust
use ironic::distributed::queue::{InMemoryQueue, QueueMessage};

let queue = InMemoryQueue::new(128); // capacity

let msg = QueueMessage {
    id: "msg-001".into(),
    headers: BTreeMap::new(),
    payload: b"work".to_vec(),
};

queue.enqueue(msg).await.unwrap();
```

## Feature flags

| Flag | Enables |
|------|---------|
| `queues` | `Queue` trait, `QueueMessage`, `QueueError`, `InMemoryQueue` |

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Queue capacity exhausted | Pass a larger capacity to `InMemoryQueue::new(capacity)` |
| Message redelivery loops | Set a delivery count in message headers; reject without requeue after threshold |
