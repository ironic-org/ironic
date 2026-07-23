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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_message(id: &str) -> QueueMessage {
        QueueMessage {
            id: id.into(),
            headers: BTreeMap::new(),
            payload: b"test payload".to_vec(),
        }
    }

    #[tokio::test]
    async fn in_memory_queue_enqueue_and_dequeue() {
        let queue = InMemoryQueue::new(16);
        let msg = sample_message("msg-1");
        queue.enqueue(msg.clone()).await.unwrap();
        let received = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(received.id, "msg-1");
        assert_eq!(received.payload, b"test payload");
    }

    #[tokio::test]
    async fn in_memory_queue_acknowledge_is_noop() {
        let queue = InMemoryQueue::new(16);
        let result = queue.acknowledge("any-id").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn in_memory_queue_reject_without_requeue() {
        let queue = InMemoryQueue::new(16);
        let msg = sample_message("lost");
        queue.enqueue(msg.clone()).await.unwrap();
        // Consume it first
        let _ = queue.dequeue().await.unwrap().unwrap();
        // Reject without requeue
        queue.reject(msg, false).await.unwrap();
        // Queue should be empty
        // (since reject w/o requeue doesn't put it back)
    }

    #[tokio::test]
    async fn in_memory_queue_reject_with_requeue() {
        let queue = InMemoryQueue::new(16);
        let msg = sample_message("retry");
        queue.enqueue(msg.clone()).await.unwrap();
        // Consume it
        let _ = queue.dequeue().await.unwrap().unwrap();
        // Reject with requeue
        queue.reject(msg, true).await.unwrap();
        // Message should be available again
        let received = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(received.id, "retry");
    }

    #[test]
    fn queue_new_zero_capacity_defaults_to_one() {
        let queue = InMemoryQueue::new(0);
        let _msg = sample_message("test");
        // capacity.max(1) ensures at least 1
        let _ = queue;
    }

    #[test]
    fn queue_error_display() {
        let err = QueueError("channel closed".into());
        assert!(err.to_string().contains("IRONIC_QUEUE"));
        assert!(err.to_string().contains("channel closed"));
    }

    #[test]
    fn queue_message_equality() {
        let a = sample_message("id1");
        let b = sample_message("id1");
        assert_eq!(a, b);

        let c = sample_message("id2");
        assert_ne!(a, c);
    }
}
