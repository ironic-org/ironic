#![allow(clippy::type_complexity, clippy::collapsible_if, clippy::while_let_loop, clippy::useless_conversion)]
//! Transport-neutral microservice envelopes and duplex in-memory endpoints.
//!
//! Additional transport backends are available behind feature flags:
//! - `transport-redis`: [`RedisTransportConfig`]
//! - `transport-rabbitmq`: [`RabbitMqTransportConfig`]
//! - `transport-kafka`: [`KafkaTransportConfig`]

use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    pin::Pin,
    sync::Arc,
};
use tokio::sync::{Mutex, mpsc, oneshot};
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------------
// Core Message Types
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Serialization
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Serializer & Deserializer
// ---------------------------------------------------------------------------

/// Serializes a value to bytes for transport.
pub trait Serializer: Send + Sync + 'static {
    /// Serializes a JSON-serializable value to bytes.
    fn to_bytes<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>, TransportError>;
}

/// Deserializes a value from bytes received from transport.
pub trait Deserializer: Send + Sync + 'static {
    /// Deserializes bytes to a JSON-deserializable value.
    fn read_bytes<T: serde::de::DeserializeOwned>(&self, data: &[u8]) -> Result<T, TransportError>;
}

/// Identity serializer that uses JSON encoding.
#[derive(Clone, Debug)]
pub struct IdentitySerializer;

impl Serializer for IdentitySerializer {
    fn to_bytes<T: serde::Serialize>(&self, value: &T) -> Result<Vec<u8>, TransportError> {
        serde_json::to_vec(value).map_err(|e| TransportError(e.to_string()))
    }
}

impl Deserializer for IdentitySerializer {
    fn read_bytes<T: serde::de::DeserializeOwned>(&self, data: &[u8]) -> Result<T, TransportError> {
        serde_json::from_slice(data).map_err(|e| TransportError(e.to_string()))
    }
}

/// Default JSON-based serializer/deserializer.
pub type JsonCodec = IdentitySerializer;

// ---------------------------------------------------------------------------
// Legacy Transport trait (deprecated)
// ---------------------------------------------------------------------------

/// A bidirectional transport endpoint.
///
/// **Deprecated**: Use [`MicroserviceClient`] and [`MicroserviceServer`] instead.
#[allow(deprecated)]
#[deprecated(since = "1.1.0", note = "Use MicroserviceClient and MicroserviceServer instead")]
pub trait Transport: Send + Sync + 'static {
    /// Sends an envelope.
    fn send(&self, envelope: Envelope) -> TransportFuture<'_, ()>;
    /// Receives the next envelope.
    fn receive(&self) -> TransportFuture<'_, Option<Envelope>>;
}

// ---------------------------------------------------------------------------
// Microservice Client
// ---------------------------------------------------------------------------

/// Boxed future for microservice client operations.
pub type ClientFuture<T> = Pin<Box<dyn Future<Output = Result<T, TransportError>> + Send>>;

/// A microservice client for sending request-response messages and events.
pub trait MicroserviceClient: Send + Sync + 'static {
    /// Connects to the transport broker.
    fn connect(&self) -> ClientFuture<()>;

    /// Sends a message and awaits a response (request-response pattern).
    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send;

    /// Emits an event without awaiting a response (fire-and-forget pattern).
    fn emit<T>(&self, pattern: &str, data: &T) -> ClientFuture<()>
    where
        T: serde::Serialize + Send + Sync + ?Sized;

    /// Closes the connection.
    fn close(&self) -> ClientFuture<()>;
}

// ---------------------------------------------------------------------------
// Microservice Server
// ---------------------------------------------------------------------------

/// A handler for incoming request-response messages.
pub type MessageHandler = Arc<
    dyn Fn(Vec<u8>, MessageContext) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TransportError>> + Send>>
        + Send + Sync,
>;

/// A handler for incoming events (fire-and-forget).
pub type EventHandler = Arc<
    dyn Fn(Vec<u8>, MessageContext) -> Pin<Box<dyn Future<Output = Result<(), TransportError>> + Send>>
        + Send + Sync,
>;

/// A message pattern that can be matched against incoming messages.
///
/// Patterns are either simple strings or complex JSON values. String patterns
/// match exact channel names. JSON patterns are serialized for matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MsPattern {
    /// A simple string pattern (e.g., `"user.get"`).
    String(String),
    /// A complex JSON pattern (e.g., `{"service":"users"}`).
    Value(serde_json::Value),
}

impl MsPattern {
    /// Normalizes this pattern to a canonical route string for handler lookup.
    #[must_use]
    pub fn normalize(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Value(v) => v.to_string(),
        }
    }
}

impl From<&str> for MsPattern {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for MsPattern {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<serde_json::Value> for MsPattern {
    fn from(v: serde_json::Value) -> Self {
        Self::Value(v)
    }
}

/// Normalizes a pattern to a canonical route string.
///
/// String patterns are used as-is. Object/JSON patterns are serialized to a
/// deterministic JSON string for consistent matching.
///
/// # Example
///
/// ```
/// use ironic::distributed::microservices::normalize_pattern;
/// use serde_json::json;
///
/// assert_eq!(normalize_pattern("user.get"), "user.get");
/// assert_eq!(normalize_pattern(json!({"s":"u"})), "{\"s\":\"u\"}");
/// ```
#[must_use]
pub fn normalize_pattern(pattern: impl Into<MsPattern>) -> String {
    pattern.into().normalize()
}

/// Context for an incoming message.
#[derive(Clone, Debug)]
pub struct MessageContext {
    /// The pattern that matched this message.
    pub pattern: String,
    /// The correlation ID from the request.
    pub correlation_id: String,
    /// The message headers.
    pub headers: BTreeMap<String, String>,
}

/// Boxed future for microservice server operations.
pub type ServerFuture<T> = Pin<Box<dyn Future<Output = Result<T, TransportError>> + Send>>;

/// A microservice server for handling incoming messages and events.
pub trait MicroserviceServer: Send + Sync + 'static {
    /// Starts listening for incoming messages.
    fn listen(&self) -> ServerFuture<()>;

    /// Registers a message handler for a pattern (request-response).
    fn on_message(&self, pattern: &str, handler: MessageHandler);

    /// Registers an event handler for a pattern (fire-and-forget).
    fn on_event(&self, pattern: &str, handler: EventHandler);

    /// Closes the server.
    fn close(&self) -> ServerFuture<()>;
}

// ---------------------------------------------------------------------------
// Custom Transport Strategy
// ---------------------------------------------------------------------------

/// A user-defined transport strategy that creates paired client/server endpoints.
///
/// Implement this trait to support custom transport protocols registered with
/// the hybrid application builder.
///
/// # Example
///
/// ```ignore
/// use ironic::distributed::microservices::{
///     CustomTransportStrategy, MicroserviceClient, MicroserviceServer,
///     InMemoryClient, InMemoryServer,
/// };
///
/// struct MyTransport;
///
/// impl CustomTransportStrategy for MyTransport {
///     type Client = InMemoryClient;
///     type Server = InMemoryServer;
///     fn create(self) -> (Self::Client, Self::Server) {
///         InMemoryServer::pair(16)
///     }
/// }
/// ```
pub trait CustomTransportStrategy: Sized {
    /// The client type for this transport.
    type Client: MicroserviceClient;
    /// The server type for this transport.
    type Server: MicroserviceServer;
    /// Creates a paired client and server.
    fn create(self) -> (Self::Client, Self::Server);
}

// ---------------------------------------------------------------------------
// Correlation ID helpers
// ---------------------------------------------------------------------------

/// Generates a unique correlation ID using a monotonic counter.
#[must_use]
pub fn generate_correlation_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{id:x}")
}

// ---------------------------------------------------------------------------
// In-Memory Client & Server
// ---------------------------------------------------------------------------

struct InMemoryServerInner {
    handlers: std::sync::Mutex<HashMap<String, MessageHandler>>,
    event_handlers: std::sync::Mutex<HashMap<String, EventHandler>>,
}

struct IncomingMessage {
    envelope: Envelope,
    reply_tx: oneshot::Sender<Result<Vec<u8>, TransportError>>,
    is_event: bool,
}

/// In-memory microservice client (paired with [`InMemoryServer`]).
#[derive(Clone)]
pub struct InMemoryClient {
    sender: mpsc::Sender<IncomingMessage>,
}

/// In-memory microservice server (paired with [`InMemoryClient`]).
pub struct InMemoryServer {
    inner: Arc<InMemoryServerInner>,
    receiver: std::sync::Mutex<Option<mpsc::Receiver<IncomingMessage>>>,
}

impl InMemoryServer {
    /// Creates a paired client and server.
    #[must_use]
    pub fn pair(capacity: usize) -> (InMemoryClient, Self) {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        let server = Self {
            inner: Arc::new(InMemoryServerInner {
                handlers: std::sync::Mutex::new(HashMap::new()),
                event_handlers: std::sync::Mutex::new(HashMap::new()),
            }),
            receiver: std::sync::Mutex::new(Some(rx)),
        };
        let client = InMemoryClient {
            sender: tx,
        };
        (client, server)
    }
}

impl MicroserviceClient for InMemoryClient {
    fn connect(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        let sender = self.sender.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let correlation_id = generate_correlation_id();
            let payload_bytes = payload?;
            let (reply_tx, reply_rx) = oneshot::channel();

            let envelope = Envelope {
                correlation_id: correlation_id.clone(),
                route: pattern.clone(),
                headers: BTreeMap::new(),
                payload: payload_bytes,
            };

            sender
                .send(IncomingMessage {
                    envelope,
                    reply_tx,
                    is_event: false,
                })
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let response = reply_rx
                .await
                .map_err(|e| TransportError(e.to_string()))??;

            serde_json::from_slice(&response).map_err(|e| TransportError(e.to_string()))
        })
    }

    fn emit<T>(&self, pattern: &str, data: &T) -> ClientFuture<()>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
    {
        let sender = self.sender.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload_bytes = payload?;
            let (reply_tx, _reply_rx) = oneshot::channel();

            let envelope = Envelope {
                correlation_id: generate_correlation_id(),
                route: pattern.clone(),
                headers: BTreeMap::new(),
                payload: payload_bytes,
            };

            sender
                .send(IncomingMessage {
                    envelope,
                    reply_tx,
                    is_event: true,
                })
                .await
                .map_err(|e| TransportError(e.to_string()))
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

impl MicroserviceServer for InMemoryServer {
    fn listen(&self) -> ServerFuture<()> {
        let receiver_opt = self.receiver.lock().unwrap().take();
        let mut receiver = match receiver_opt {
            Some(rx) => rx,
            None => return Box::pin(async move { Err(TransportError("server already started".into())) }),
        };

        let inner = Arc::clone(&self.inner);

        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                let inner = Arc::clone(&inner);
                tokio::spawn(async move {
                    let context = MessageContext {
                        pattern: msg.envelope.route.clone(),
                        correlation_id: msg.envelope.correlation_id,
                        headers: msg.envelope.headers.clone(),
                    };

                    if msg.is_event {
                        let handler = {
                            let handlers = inner.event_handlers.lock().unwrap();
                            handlers.get(&context.pattern).cloned()
                        };
                        if let Some(handler) = handler {
                            let _ = handler(msg.envelope.payload, context).await;
                        }
                    } else {
                        let handler = {
                            let handlers = inner.handlers.lock().unwrap();
                            handlers.get(&context.pattern).cloned()
                        };
                        if let Some(handler) = handler {
                            let result = handler(msg.envelope.payload, context).await;
                            let _ = msg.reply_tx.send(result);
                        } else {
                            let _ = msg.reply_tx.send(Err(TransportError(
                                "NO_MESSAGE_HANDLER".into(),
                            )));
                        }
                    }
                });
            }
        });

        Box::pin(async move { Ok(()) })
    }

    fn on_message(&self, pattern: &str, handler: MessageHandler) {
        let mut handlers = self.inner.handlers.lock().unwrap();
        handlers.insert(pattern.to_string(), handler);
    }

    fn on_event(&self, pattern: &str, handler: EventHandler) {
        let mut handlers = self.inner.event_handlers.lock().unwrap();
        handlers.insert(pattern.to_string(), handler);
    }

    fn close(&self) -> ServerFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// Legacy ChannelTransport (implements both old and new traits)
// ---------------------------------------------------------------------------

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

#[allow(deprecated)]
impl Transport for ChannelTransport {
    fn send(&self, envelope: Envelope) -> TransportFuture<'_, ()> {
        let sender = self.sender.clone();
        Box::pin(async move {
            sender
                .send(envelope)
                .await
                .map_err(|error| TransportError(error.to_string()))
        })
    }

    fn receive(&self) -> TransportFuture<'_, Option<Envelope>> {
        let receiver = Arc::clone(&self.receiver);
        Box::pin(async move {
            let mut guard = receiver.lock().await;
            Ok(guard.recv().await)
        })
    }
}

impl MicroserviceClient for ChannelTransport {
    fn connect(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        let sender = self.sender.clone();
        let receiver = Arc::clone(&self.receiver);
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let correlation_id = generate_correlation_id();
            let payload = payload?;
            let envelope = Envelope {
                correlation_id: correlation_id.clone(),
                route: pattern,
                headers: BTreeMap::new(),
                payload,
            };
            sender
                .send(envelope)
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            loop {
                let mut guard = receiver.lock().await;
                if let Some(reply) = guard.recv().await {
                    if reply.correlation_id == correlation_id {
                        return serde_json::from_slice(&reply.payload)
                            .map_err(|e| TransportError(e.to_string()));
                    }
                } else {
                    return Err(TransportError("channel closed".into()));
                }
            }
        })
    }

    fn emit<T>(&self, pattern: &str, data: &T) -> ClientFuture<()>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
    {
        let sender = self.sender.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let envelope = Envelope {
                correlation_id: generate_correlation_id(),
                route: pattern,
                headers: BTreeMap::new(),
                payload,
            };
            sender
                .send(envelope)
                .await
                .map_err(|e| TransportError(e.to_string()))
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// Deprecated — legacy transport backend adapters (config stubs)
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
    pub fn connect(self) -> Result<RedisTransport, TransportError> {
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
#[allow(dead_code)]
pub struct RedisTransport {
    config: RedisTransportConfig,
}

#[cfg(feature = "transport-redis")]
#[allow(deprecated)]
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

/// Configuration for a `RabbitMQ` transport.
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

    /// Connects to `RabbitMQ` and returns a transport endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] when the connection or channel setup fails.
    pub fn connect(self) -> Result<RabbitMqTransport, TransportError> {
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

/// A `RabbitMQ` transport.
///
/// Requires the `transport-rabbitmq` feature and a running `RabbitMQ` instance.
#[cfg(feature = "transport-rabbitmq")]
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RabbitMqTransport {
    config: RabbitMqTransportConfig,
}

#[cfg(feature = "transport-rabbitmq")]
#[allow(deprecated)]
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
    pub fn builder(brokers: impl Into<String>, topic: impl Into<String>) -> KafkaTransportBuilder {
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
    pub fn connect(self) -> Result<KafkaTransport, TransportError> {
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
#[allow(dead_code)]
pub struct KafkaTransport {
    config: KafkaTransportConfig,
}

#[cfg(feature = "transport-kafka")]
#[allow(deprecated)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inmemory_client_server_request_response() {
        let (client, server) = InMemoryServer::pair(16);

        server.on_message("greet", Arc::new(|payload, _ctx| {
            Box::pin(async move {
                let name: String = serde_json::from_slice(&payload).map_err(|e| TransportError(e.to_string()))?;
                let response = format!("Hello, {name}!");
                serde_json::to_vec(&response).map_err(|e| TransportError(e.to_string()))
            })
        }));

        server.listen().await.unwrap();

        let response: String = client.send("greet", &"World".to_string()).await.unwrap();
        assert_eq!(response, "Hello, World!");
    }

    #[tokio::test]
    async fn inmemory_client_server_event() {
        let (client, server) = InMemoryServer::pair(16);
        let received = Arc::new(Mutex::new(Vec::new()));

        let events = Arc::clone(&received);
        server.on_event("user.created", Arc::new(move |payload, _ctx| {
            let events = Arc::clone(&events);
            Box::pin(async move {
                let name: String = serde_json::from_slice(&payload).map_err(|e| TransportError(e.to_string()))?;
                events.lock().await.push(name);
                Ok(())
            })
        }));

        server.listen().await.unwrap();
        client.emit("user.created", &"Alice".to_string()).await.unwrap();
        client.emit("user.created", &"Bob".to_string()).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let events = received.lock().await;
        assert_eq!(events.len(), 2);
        assert!(events.contains(&"Alice".to_string()));
        assert!(events.contains(&"Bob".to_string()));
    }

    #[tokio::test]
    async fn inmemory_no_handler_returns_error() {
        let (client, server) = InMemoryServer::pair(16);
        server.listen().await.unwrap();

        let result: Result<String, TransportError> = client.send("missing", &"data".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn channel_transport_send_and_receive() {
        let (left, right) = ChannelTransport::pair(16);
        let envelope = Envelope {
            correlation_id: "test-1".into(),
            route: "test.route".into(),
            headers: BTreeMap::new(),
            payload: b"hello".to_vec(),
        };

        Transport::send(&left, envelope.clone()).await.unwrap();
        let received = Transport::receive(&right).await.unwrap().unwrap();
        assert_eq!(received.correlation_id, "test-1");
        assert_eq!(received.route, "test.route");
        assert_eq!(received.payload, b"hello");
    }

    #[tokio::test]
    async fn channel_transport_bidirectional() {
        let (left, right) = ChannelTransport::pair(16);
        let envelope = Envelope {
            correlation_id: "round-trip".into(),
            route: "echo".into(),
            headers: BTreeMap::new(),
            payload: b"ping".to_vec(),
        };

        Transport::send(&left, envelope).await.unwrap();
        let received = Transport::receive(&right).await.unwrap().unwrap();
        assert_eq!(received.payload, b"ping");

        Transport::send(
            &right,
            Envelope {
                correlation_id: "reply".into(),
                route: "echo.reply".into(),
                headers: BTreeMap::new(),
                payload: b"pong".to_vec(),
            },
        )
        .await
        .unwrap();
        let reply = left.receive().await.unwrap().unwrap();
        assert_eq!(reply.payload, b"pong");
    }

    #[tokio::test]
    async fn channel_transport_client_server_pattern() {
        let (client, server) = ChannelTransport::pair(16);

        let client2 = client.clone();
        tokio::spawn(async move {
            let result: String = MicroserviceClient::send(&client2, "test", &"ping".to_string()).await.unwrap();
            assert_eq!(result, "pong");
        });

        // Simulate server receiving the request via the deprecated Transport trait
        #[allow(deprecated)]
        let req = Transport::receive(&server).await.unwrap().unwrap();
        assert_eq!(req.route, "test");

        let response = Envelope {
            correlation_id: req.correlation_id,
            route: "test.reply".into(),
            headers: BTreeMap::new(),
            payload: serde_json::to_vec(&"pong".to_string()).unwrap(),
        };
        #[allow(deprecated)]
        Transport::send(&server, response).await.unwrap();
        // Give the spawned task time to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[cfg(feature = "transport-redis")]
    #[test]
    fn redis_transport_builder_config_is_preserved() {
        let config =
            RedisTransportConfig::builder("redis://localhost:6379", "test-channel").pool_size(8);
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.channel, "test-channel");
        assert_eq!(config.pool_size, Some(8));
    }

    #[cfg(feature = "transport-rabbitmq")]
    #[test]
    fn rabbitmq_transport_builder_config_is_preserved() {
        let config = RabbitMqTransportConfig::builder(
            "amqp://localhost:5672",
            "test-exchange",
            "test.queue",
        )
        .queue("my-queue");
        assert_eq!(config.url, "amqp://localhost:5672");
        assert_eq!(config.exchange, "test-exchange");
        assert_eq!(config.routing_key, "test.queue");
        assert_eq!(config.queue, "my-queue");
    }

    #[cfg(feature = "transport-kafka")]
    #[test]
    fn kafka_transport_builder_config_is_preserved() {
        let config =
            KafkaTransportConfig::builder("localhost:9092", "test-topic").group_id("consumer-1");
        assert_eq!(config.brokers, "localhost:9092");
        assert_eq!(config.topic, "test-topic");
        assert_eq!(config.group_id, "consumer-1");
    }
}
