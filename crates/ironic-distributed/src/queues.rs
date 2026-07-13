//! Backend-neutral queues and a bounded in-memory implementation.

use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};
use tokio::sync::{Mutex, mpsc};

/// A queue message with application headers and opaque payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueMessage {
    /// Application-assigned unique message identifier.
    pub id: String,
    /// Routing or tracing headers.
    pub headers: BTreeMap<String, String>,
    /// Serialized message payload.
    pub payload: Vec<u8>,
}

/// A queue failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("IRONIC_QUEUE: {0}")]
pub struct QueueError(pub String);

/// Boxed queue operation.
pub type QueueFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, QueueError>> + Send + 'a>>;

/// Asynchronous at-least-once queue contract.
pub trait Queue: Send + Sync + 'static {
    /// Enqueues a message.
    fn enqueue(&self, message: QueueMessage) -> QueueFuture<'_, ()>;
    /// Waits for the next message.
    fn dequeue(&self) -> QueueFuture<'_, Option<QueueMessage>>;
    /// Acknowledges successful processing.
    fn acknowledge<'a>(&'a self, message_id: &'a str) -> QueueFuture<'a, ()>;
    /// Rejects processing, optionally requesting redelivery.
    fn reject(&self, message: QueueMessage, requeue: bool) -> QueueFuture<'_, ()>;
}

/// Bounded process-local queue for tests, development, and single-process workers.
#[derive(Clone)]
pub struct InMemoryQueue {
    sender: mpsc::Sender<QueueMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<QueueMessage>>>,
}

impl InMemoryQueue {
    /// Creates a bounded queue.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity.max(1));
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
}

impl Queue for InMemoryQueue {
    fn enqueue(&self, message: QueueMessage) -> QueueFuture<'_, ()> {
        Box::pin(async move {
            self.sender
                .send(message)
                .await
                .map_err(|error| QueueError(error.to_string()))
        })
    }

    fn dequeue(&self) -> QueueFuture<'_, Option<QueueMessage>> {
        Box::pin(async move { Ok(self.receiver.lock().await.recv().await) })
    }

    fn acknowledge<'a>(&'a self, _message_id: &'a str) -> QueueFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }

    fn reject(&self, message: QueueMessage, requeue: bool) -> QueueFuture<'_, ()> {
        Box::pin(async move {
            if requeue {
                self.enqueue(message).await?;
            }
            Ok(())
        })
    }
}
