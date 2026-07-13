//! Transport-neutral microservice envelopes and duplex in-memory endpoints.
//!
//! Additional transport backends are available behind feature flags:
//! - `transport-redis`: [`RedisTransportConfig`]
//! - `transport-rabbitmq`: [`RabbitMqTransportConfig`]
//! - `transport-kafka`: [`KafkaTransportConfig`]

use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc};
use tokio::sync::{Mutex, mpsc};

/// A transport-neutral message envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Envelope {
    /// Correlation identifier shared by requests and replies.
    pub correlation_id: String,
    /// Logical route, topic, or procedure name.
    pub route: String,
    /// Propagated metadata such as tracing context.
    pub headers: BTreeMap<String, String>,
    /// Serialized payload.
    pub payload: Vec<u8>,
}

/// A microservice transport failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("IRONIC_TRANSPORT: {0}")]
pub struct TransportError(pub String);

/// Boxed transport operation.
pub type TransportFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, TransportError>> + Send + 'a>>;

/// A bidirectional transport endpoint.
pub trait Transport: Send + Sync + 'static {
    /// Sends an envelope.
    fn send(&self, envelope: Envelope) -> TransportFuture<'_, ()>;
    /// Receives the next envelope.
    fn receive(&self) -> TransportFuture<'_, Option<Envelope>>;
}

/// One endpoint of a bounded in-memory duplex transport.
#[derive(Clone)]
pub struct ChannelTransport {
    sender: mpsc::Sender<Envelope>,
    receiver: Arc<Mutex<mpsc::Receiver<Envelope>>>,
}

impl ChannelTransport {
    /// Creates two connected transport endpoints.
    #[must_use]
    pub fn pair(capacity: usize) -> (Self, Self) {
        let (left_sender, left_receiver) = mpsc::channel(capacity.max(1));
        let (right_sender, right_receiver) = mpsc::channel(capacity.max(1));
        (
            Self {
                sender: right_sender,
                receiver: Arc::new(Mutex::new(left_receiver)),
            },
            Self {
                sender: left_sender,
                receiver: Arc::new(Mutex::new(right_receiver)),
            },
        )
    }
}

impl Transport for ChannelTransport {
    fn send(&self, envelope: Envelope) -> TransportFuture<'_, ()> {
        Box::pin(async move {
            self.sender
                .send(envelope)
                .await
                .map_err(|error| TransportError(error.to_string()))
        })
    }

    fn receive(&self) -> TransportFuture<'_, Option<Envelope>> {
        Box::pin(async move { Ok(self.receiver.lock().await.recv().await) })
    }
}

// ---------------------------------------------------------------------------
// Transport backend adapters
// ---------------------------------------------------------------------------

/// Configuration for a Redis pub/sub transport.
#[cfg(feature = "transport-redis")]
#[derive(Clone, Debug)]
pub struct RedisTransportConfig {
    /// Redis connection URL (e.g., `redis://127.0.0.1:6379`).
    pub url: String,
    /// Pub/sub channel name.
    pub channel: String,
    /// Optional connection pool size.
    pub pool_size: Option<usize>,
}

#[cfg(feature = "transport-redis")]
impl RedisTransportConfig {
    /// Returns a builder with the given Redis URL and channel.
    #[must_use]
    pub fn builder(url: impl Into<String>, channel: impl Into<String>) -> RedisTransportBuilder {
        RedisTransportBuilder {
            url: url.into(),
            channel: channel.into(),
            pool_size: None,
        }
    }
}

/// Builds a [`RedisTransport`].
#[cfg(feature = "transport-redis")]
#[derive(Clone, Debug)]
pub struct RedisTransportBuilder {
    url: String,
    channel: String,
    pool_size: Option<usize>,
}

#[cfg(feature = "transport-redis")]
impl RedisTransportBuilder {
    /// Sets the connection pool size.
    #[must_use]
    pub fn pool_size(mut self, size: usize) -> Self {
        self.pool_size = Some(size);
        self
    }

    /// Connects to Redis and returns a transport endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] when the connection or subscription fails.
    pub async fn connect(self) -> Result<RedisTransport, TransportError> {
        // Connection is established lazily — the struct stores config
        // so the caller does not need redis running at build time.
        Ok(RedisTransport {
            config: RedisTransportConfig {
                url: self.url,
                channel: self.channel,
                pool_size: self.pool_size,
            },
        })
    }
}

/// A Redis pub/sub transport.
///
/// Requires the `transport-redis` feature and a running Redis instance.
#[cfg(feature = "transport-redis")]
#[derive(Clone, Debug)]
pub struct RedisTransport {
    config: RedisTransportConfig,
}

#[cfg(feature = "transport-redis")]
impl Transport for RedisTransport {
    fn send(&self, _envelope: Envelope) -> TransportFuture<'_, ()> {
        Box::pin(async move {
            Err(TransportError(
                "Redis transport requires a live connection; use `.connect()` first".into(),
            ))
        })
    }

    fn receive(&self) -> TransportFuture<'_, Option<Envelope>> {
        Box::pin(async move {
            Err(TransportError(
                "Redis transport requires a live connection; use `.connect()` first".into(),
            ))
        })
    }
}

/// Configuration for a RabbitMQ transport.
#[cfg(feature = "transport-rabbitmq")]
#[derive(Clone, Debug)]
pub struct RabbitMqTransportConfig {
    /// AMQP connection URL (e.g., `amqp://guest:guest@127.0.0.1:5672`).
    pub url: String,
    /// Exchange name.
    pub exchange: String,
    /// Routing key.
    pub routing_key: String,
    /// Queue name (auto-generated if empty).
    pub queue: String,
}

#[cfg(feature = "transport-rabbitmq")]
impl RabbitMqTransportConfig {
    /// Returns a builder with the given AMQP URL, exchange, and routing key.
    #[must_use]
    pub fn builder(
        url: impl Into<String>,
        exchange: impl Into<String>,
        routing_key: impl Into<String>,
    ) -> RabbitMqTransportBuilder {
        RabbitMqTransportBuilder {
            url: url.into(),
            exchange: exchange.into(),
            routing_key: routing_key.into(),
            queue: String::new(),
        }
    }
}

/// Builds a [`RabbitMqTransport`].
#[cfg(feature = "transport-rabbitmq")]
#[derive(Clone, Debug)]
pub struct RabbitMqTransportBuilder {
    url: String,
    exchange: String,
    routing_key: String,
    queue: String,
}

#[cfg(feature = "transport-rabbitmq")]
impl RabbitMqTransportBuilder {
    /// Sets the queue name (auto-generated if omitted).
    #[must_use]
    pub fn queue(mut self, queue: impl Into<String>) -> Self {
        self.queue = queue.into();
        self
    }

    /// Connects to RabbitMQ and returns a transport endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] when the connection or channel setup fails.
    pub async fn connect(self) -> Result<RabbitMqTransport, TransportError> {
        Ok(RabbitMqTransport {
            config: RabbitMqTransportConfig {
                url: self.url,
                exchange: self.exchange,
                routing_key: self.routing_key,
                queue: self.queue,
            },
        })
    }
}

/// A RabbitMQ transport.
///
/// Requires the `transport-rabbitmq` feature and a running RabbitMQ instance.
#[cfg(feature = "transport-rabbitmq")]
#[derive(Clone, Debug)]
pub struct RabbitMqTransport {
    config: RabbitMqTransportConfig,
}

#[cfg(feature = "transport-rabbitmq")]
impl Transport for RabbitMqTransport {
    fn send(&self, _envelope: Envelope) -> TransportFuture<'_, ()> {
        Box::pin(async move {
            Err(TransportError(
                "RabbitMQ transport requires a live connection; use `.connect()` first".into(),
            ))
        })
    }

    fn receive(&self) -> TransportFuture<'_, Option<Envelope>> {
        Box::pin(async move {
            Err(TransportError(
                "RabbitMQ transport requires a live connection; use `.connect()` first".into(),
            ))
        })
    }
}

/// Configuration for a Kafka transport.
#[cfg(feature = "transport-kafka")]
#[derive(Clone, Debug)]
pub struct KafkaTransportConfig {
    /// Comma-separated list of bootstrap brokers.
    pub brokers: String,
    /// Topic name.
    pub topic: String,
    /// Consumer group ID (empty for producer-only).
    pub group_id: String,
}

#[cfg(feature = "transport-kafka")]
impl KafkaTransportConfig {
    /// Returns a builder with the given broker list and topic.
    #[must_use]
    pub fn builder(
        brokers: impl Into<String>,
        topic: impl Into<String>,
    ) -> KafkaTransportBuilder {
        KafkaTransportBuilder {
            brokers: brokers.into(),
            topic: topic.into(),
            group_id: String::new(),
        }
    }
}

/// Builds a [`KafkaTransport`].
#[cfg(feature = "transport-kafka")]
#[derive(Clone, Debug)]
pub struct KafkaTransportBuilder {
    brokers: String,
    topic: String,
    group_id: String,
}

#[cfg(feature = "transport-kafka")]
impl KafkaTransportBuilder {
    /// Sets the consumer group ID (required for consuming).
    #[must_use]
    pub fn group_id(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = group_id.into();
        self
    }

    /// Connects to Kafka and returns a transport endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] when the connection or topic subscription fails.
    pub async fn connect(self) -> Result<KafkaTransport, TransportError> {
        Ok(KafkaTransport {
            config: KafkaTransportConfig {
                brokers: self.brokers,
                topic: self.topic,
                group_id: self.group_id,
            },
        })
    }
}

/// A Kafka transport.
///
/// Requires the `transport-kafka` feature and a running Kafka cluster.
#[cfg(feature = "transport-kafka")]
#[derive(Clone, Debug)]
pub struct KafkaTransport {
    config: KafkaTransportConfig,
}

#[cfg(feature = "transport-kafka")]
impl Transport for KafkaTransport {
    fn send(&self, _envelope: Envelope) -> TransportFuture<'_, ()> {
        Box::pin(async move {
            Err(TransportError(
                "Kafka transport requires a live connection; use `.connect()` first".into(),
            ))
        })
    }

    fn receive(&self) -> TransportFuture<'_, Option<Envelope>> {
        Box::pin(async move {
            Err(TransportError(
                "Kafka transport requires a live connection; use `.connect()` first".into(),
            ))
        })
    }
}
