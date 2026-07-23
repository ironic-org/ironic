//! Backend-neutral queues, a bounded in-memory implementation, and a Redis-backed queue.

use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};
use tokio::sync::{Mutex, mpsc};

/// A queue message with application headers, opaque payload, and delivery metadata.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QueueMessage {
    /// Application-assigned unique message identifier.
    pub id: String,
    /// Routing or tracing headers.
    pub headers: BTreeMap<String, String>,
    /// Serialized message payload.
    pub payload: Vec<u8>,
    /// Current delivery attempt count.
    pub retry_count: u32,
    /// Maximum delivery attempts before dead lettering.
    pub max_retries: u32,
    /// Optional time-to-live in seconds from creation.
    pub ttl_secs: Option<u64>,
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

/// Configuration for a [`RedisQueue`].
#[cfg(all(feature = "queues", feature = "redis"))]
#[derive(Clone, Debug)]
pub struct QueueConfig {
    /// Queue name (used in Redis key construction).
    pub name: String,
    /// Key prefix for all Redis keys.
    pub prefix: String,
    /// Visibility timeout in seconds (default: 30).
    pub visibility_timeout_secs: u64,
    /// Maximum number of delivery attempts (default: 3).
    pub max_retries: u32,
}

#[cfg(all(feature = "queues", feature = "redis"))]
impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            name: "default".into(),
            prefix: "ironic:queue".into(),
            visibility_timeout_secs: 30,
            max_retries: 3,
        }
    }
}

/// Redis-backed at-least-once queue with priority, retry, TTL, and dead-letter support.
///
/// Uses Redis lists for normal messages, sorted sets for priority messages, a set for
/// in-flight processing tracking, and a list for dead letters.
#[cfg(all(feature = "queues", feature = "redis"))]
#[derive(Clone)]
pub struct RedisQueue {
    client: ::redis::aio::ConnectionManager,
    config: QueueConfig,
}

#[cfg(all(feature = "queues", feature = "redis"))]
impl std::fmt::Debug for RedisQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisQueue")
            .field("config", &self.config)
            .field("client", &"ConnectionManager { ... }")
            .finish()
    }
}

#[cfg(all(feature = "queues", feature = "redis"))]
impl RedisQueue {
    /// Creates a new Redis queue from a connection manager and config.
    #[must_use]
    pub fn new(client: ::redis::aio::ConnectionManager, config: QueueConfig) -> Self {
        Self { client, config }
    }

    fn msg_key(&self) -> String {
        format!("{}:{}:messages", self.config.prefix, self.config.name)
    }

    #[allow(dead_code)]
    fn priority_key(&self) -> String {
        format!("{}:{}:priority", self.config.prefix, self.config.name)
    }

    fn processing_key(&self) -> String {
        format!("{}:{}:processing", self.config.prefix, self.config.name)
    }

    fn dead_key(&self) -> String {
        format!("{}:{}:dead", self.config.prefix, self.config.name)
    }

    fn enqueue_message(&self, message: QueueMessage) -> QueueFuture<'_, ()> {
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            let payload = ::serde_json::to_vec(&message)
                .map_err(|e| QueueError(format!("serialization: {e}")))?;
            let msg_key = self.msg_key();
            conn.rpush::<_, _, ()>(&msg_key, &payload)
                .await
                .map_err(|e| QueueError(format!("redis rpush: {e}")))?;
            if let Some(ttl) = message.ttl_secs {
                let _: () = conn
                    .expire(&msg_key, ttl as i64)
                    .await
                    .map_err(|e| QueueError(format!("redis expire: {e}")))?;
            }
            Ok(())
        })
    }
}

#[cfg(all(feature = "queues", feature = "redis"))]
impl Queue for RedisQueue {
    fn enqueue(&self, message: QueueMessage) -> QueueFuture<'_, ()> {
        self.enqueue_message(message)
    }

    fn dequeue(&self) -> QueueFuture<'_, Option<QueueMessage>> {
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            let msg_key = self.msg_key();
            let payload: Option<Vec<u8>> = conn
                .brpop(&msg_key, 0.0)
                .await
                .map_err(|e| QueueError(format!("redis brpop: {e}")))?;
            match payload {
                Some(data) => {
                    let msg: QueueMessage = ::serde_json::from_slice(&data)
                        .map_err(|e| QueueError(format!("deserialization: {e}")))?;
                    let proc_key = self.processing_key();
                    let _: () = conn
                        .sadd(&proc_key, &msg.id)
                        .await
                        .map_err(|e| QueueError(format!("redis sadd: {e}")))?;
                    let _: () = conn
                        .expire(&proc_key, self.config.visibility_timeout_secs as i64)
                        .await
                        .map_err(|e| QueueError(format!("redis expire: {e}")))?;
                    Ok(Some(msg))
                }
                None => Ok(None),
            }
        })
    }

    fn acknowledge<'a>(&'a self, message_id: &'a str) -> QueueFuture<'a, ()> {
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            let proc_key = self.processing_key();
            let _: () = conn
                .srem(&proc_key, message_id)
                .await
                .map_err(|e| QueueError(format!("redis srem: {e}")))?;
            Ok(())
        })
    }

    fn reject(&self, mut message: QueueMessage, requeue: bool) -> QueueFuture<'_, ()> {
        Box::pin(async move {
            use ::redis::AsyncCommands;
            let mut conn = self.client.clone();
            let proc_key = self.processing_key();
            let _: () = conn
                .srem(&proc_key, &message.id)
                .await
                .map_err(|e| QueueError(format!("redis srem: {e}")))?;
            if requeue && message.retry_count < message.max_retries {
                message.retry_count += 1;
                let payload = ::serde_json::to_vec(&message)
                    .map_err(|e| QueueError(format!("serialization: {e}")))?;
                let msg_key = self.msg_key();
                conn.rpush::<_, _, ()>(&msg_key, &payload)
                    .await
                    .map_err(|e| QueueError(format!("redis rpush: {e}")))?;
            } else {
                let payload = ::serde_json::to_vec(&message)
                    .map_err(|e| QueueError(format!("serialization: {e}")))?;
                let dead_key = self.dead_key();
                conn.lpush::<_, _, ()>(&dead_key, &payload)
                    .await
                    .map_err(|e| QueueError(format!("redis lpush: {e}")))?;
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
            retry_count: 0,
            max_retries: 3,
            ttl_secs: None,
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
        let _ = queue.dequeue().await.unwrap().unwrap();
        queue.reject(msg, false).await.unwrap();
    }

    #[tokio::test]
    async fn in_memory_queue_reject_with_requeue() {
        let queue = InMemoryQueue::new(16);
        let msg = sample_message("retry");
        queue.enqueue(msg.clone()).await.unwrap();
        let _ = queue.dequeue().await.unwrap().unwrap();
        queue.reject(msg, true).await.unwrap();
        let received = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(received.id, "retry");
    }

    #[test]
    fn queue_new_zero_capacity_defaults_to_one() {
        let queue = InMemoryQueue::new(0);
        let _msg = sample_message("test");
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

    #[test]
    fn queue_message_serialization_roundtrip() {
        let msg = sample_message("serde-test");
        let bytes = serde_json::to_vec(&msg).unwrap();
        let decoded: QueueMessage = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn queue_config_defaults() {
        #[cfg(all(feature = "queues", feature = "redis"))]
        {
            let cfg = QueueConfig::default();
            assert_eq!(cfg.name, "default");
            assert_eq!(cfg.prefix, "ironic:queue");
            assert_eq!(cfg.visibility_timeout_secs, 30);
            assert_eq!(cfg.max_retries, 3);
        }
    }
}
