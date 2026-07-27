//! Transport-agnostic DI-managed event client and server.
//!
//! Provides [`EventClient`] and [`EventServer`] as injectable providers
//! with automatic lifecycle (connect/listen on bootstrap, close on shutdown).
//! The transport backend is selected via [`TransportConfig`].

use std::sync::Arc;

use super::microservices::{
    ClientFuture, EventHandler, InMemoryClient, InMemoryServer, MessageHandler, MicroserviceClient,
    MicroserviceServer, ServerFuture, TransportError,
};
#[cfg(feature = "transport-kafka")]
use super::transport_kafka::{KafkaClient, KafkaClientConfig, KafkaServer, KafkaServerConfig};
#[cfg(feature = "transport-redis")]
use super::transport_redis::{RedisClient, RedisClientConfig, RedisServer, RedisServerConfig};
use crate::{
    Dependency, LifecycleError, LifecycleFuture, OnApplicationBootstrap, OnApplicationShutdown,
    ProviderDefinition, Scope, ShutdownSignal,
};

// ---------------------------------------------------------------------------
// Transport selection
// ---------------------------------------------------------------------------

/// Backend transport selection.
///
/// Default varies by available features: `Kafka` > `InMemory`.
#[derive(Clone, Debug)]
pub enum TransportKind {
    /// Apache Kafka transport (requires `transport-kafka` feature).
    #[cfg(feature = "transport-kafka")]
    Kafka,
    /// Redis pub/sub transport (requires `transport-redis` feature).
    #[cfg(feature = "transport-redis")]
    Redis,
    /// Process-local in-memory transport (always available).
    InMemory,
}

#[cfg(feature = "transport-kafka")]
#[allow(clippy::derivable_impls)]
impl Default for TransportKind {
    fn default() -> Self {
        Self::Kafka
    }
}

#[cfg(not(feature = "transport-kafka"))]
impl Default for TransportKind {
    fn default() -> Self {
        Self::InMemory
    }
}

/// Single source of transport configuration.
///
/// Add this as a provider to your module and override fields via
/// `override_provider` in the application builder for environment-specific
/// configuration.
#[derive(Clone, Debug)]
pub struct TransportConfig {
    /// Which transport backend to use.
    pub kind: TransportKind,
    /// Broker / connection URL
    /// (e.g. `"127.0.0.1:9092"` for Kafka, `"redis://127.0.0.1:6379"` for Redis).
    pub brokers: String,
    /// Topic / channel name (e.g. `"exeos-events"`).
    pub topic: String,
    /// Consumer group ID (ignored by non-Kafka transports).
    pub group_id: String,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            kind: TransportKind::default(),
            brokers: std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "127.0.0.1:9092".into()),
            topic: std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "ironic-events".into()),
            group_id: std::env::var("KAFKA_GROUP_ID").unwrap_or_else(|_| "default".into()),
        }
    }
}

impl TransportConfig {
    /// Creates a [`ProviderDefinition`] that resolves from env vars by default.
    ///
    /// The provider is marked `eager` so env vars are read at startup.
    pub fn provider_definition() -> ProviderDefinition {
        ProviderDefinition::constructor::<Self, _>(
            Scope::Singleton,
            vec![],
            |_| Ok(Self::default()),
        )
        .eager()
    }
}

// ---------------------------------------------------------------------------
// TransportClient — dispatch enum (MicroserviceClient is not object-safe)
// ---------------------------------------------------------------------------

/// Internal dispatch enum wrapping all transport client implementations.
enum TransportClient {
    InMemory(InMemoryClient),
    #[cfg(feature = "transport-kafka")]
    Kafka(KafkaClient),
    #[cfg(feature = "transport-redis")]
    Redis(RedisClient),
}

impl MicroserviceClient for TransportClient {
    fn connect(&self) -> ClientFuture<()> {
        match self {
            Self::InMemory(c) => c.connect(),
            #[cfg(feature = "transport-kafka")]
            Self::Kafka(c) => c.connect(),
            #[cfg(feature = "transport-redis")]
            Self::Redis(c) => c.connect(),
        }
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        match self {
            Self::InMemory(c) => c.send(pattern, data),
            #[cfg(feature = "transport-kafka")]
            Self::Kafka(c) => c.send(pattern, data),
            #[cfg(feature = "transport-redis")]
            Self::Redis(c) => c.send(pattern, data),
        }
    }

    fn emit<T>(&self, pattern: &str, data: &T) -> ClientFuture<()>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
    {
        match self {
            Self::InMemory(c) => c.emit(pattern, data),
            #[cfg(feature = "transport-kafka")]
            Self::Kafka(c) => c.emit(pattern, data),
            #[cfg(feature = "transport-redis")]
            Self::Redis(c) => c.emit(pattern, data),
        }
    }

    fn close(&self) -> ClientFuture<()> {
        match self {
            Self::InMemory(c) => c.close(),
            #[cfg(feature = "transport-kafka")]
            Self::Kafka(c) => c.close(),
            #[cfg(feature = "transport-redis")]
            Self::Redis(c) => c.close(),
        }
    }
}

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------

fn create_transport_client(config: &TransportConfig) -> TransportClient {
    match config.kind {
        #[cfg(feature = "transport-kafka")]
        TransportKind::Kafka => TransportClient::Kafka(KafkaClient::new(KafkaClientConfig {
            brokers: config.brokers.clone(),
            topic: config.topic.clone(),
        })),
        #[cfg(feature = "transport-redis")]
        TransportKind::Redis => TransportClient::Redis(RedisClient::new(RedisClientConfig {
            url: config.brokers.clone(),
            ..Default::default()
        })),
        TransportKind::InMemory => {
            let (client, _server) = InMemoryServer::pair(16);
            TransportClient::InMemory(client)
        }
    }
}

fn create_transport_server(config: &TransportConfig) -> Arc<dyn MicroserviceServer> {
    match config.kind {
        #[cfg(feature = "transport-kafka")]
        TransportKind::Kafka => Arc::new(KafkaServer::new(KafkaServerConfig {
            brokers: config.brokers.clone(),
            topic: config.topic.clone(),
            group_id: config.group_id.clone(),
        })),
        #[cfg(feature = "transport-redis")]
        TransportKind::Redis => Arc::new(RedisServer::new(RedisServerConfig {
            url: config.brokers.clone(),
            ..Default::default()
        })),
        TransportKind::InMemory => {
            let (_client, server) = InMemoryServer::pair(16);
            Arc::new(server)
        }
    }
}

// ---------------------------------------------------------------------------
// EventClient
// ---------------------------------------------------------------------------

/// A transport-agnostic event producer with auto-connect lifecycle.
///
/// Injects [`TransportConfig`] to select the backend. Automatically connects
/// during `OnApplicationBootstrap` and closes during `OnApplicationShutdown`.
pub struct EventClient {
    client: TransportClient,
}

impl EventClient {
    /// Creates a [`ProviderDefinition`] that injects [`TransportConfig`].
    ///
    /// Register with `#[module(providers = [EventClient, ...])]`.
    pub fn provider_definition() -> ProviderDefinition {
        ProviderDefinition::factory::<Self, _, _>(
            Scope::Singleton,
            vec![Dependency::required::<TransportConfig>()],
            |resolver| async move {
                let config: Arc<TransportConfig> = resolver.resolve().await?;
                let client = create_transport_client(&config);
                Ok(Self { client })
            },
        )
        .eager()
    }

    /// Emits an event on the configured transport.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] when the serialization or transport send fails.
    pub async fn emit<T: serde::Serialize + Send + Sync>(
        &self,
        pattern: &str,
        event: &T,
    ) -> Result<(), TransportError> {
        self.client.emit(pattern, event).await
    }
}

impl OnApplicationBootstrap for EventClient {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.client
                .connect()
                .await
                .map_err(|e| LifecycleError::new(format!("EVENT_CLIENT_CONNECT: {}", e.0)))?;
            tracing::info!("event client connected");
            Ok(())
        })
    }
}

impl OnApplicationShutdown for EventClient {
    fn on_application_shutdown(&self, _signal: ShutdownSignal) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.client.close().await.ok();
            tracing::info!("event client disconnected");
            Ok(())
        })
    }
}

// ---------------------------------------------------------------------------
// EventServer
// ---------------------------------------------------------------------------

/// A transport-agnostic event consumer with auto-listen lifecycle.
///
/// Injects [`TransportConfig`] to select the backend. Implements
/// [`MicroserviceServer`] so that [`#[event_handler]`](crate::event_handler)
/// can register handlers before `listen()` starts during
/// `OnApplicationBootstrap`.
pub struct EventServer {
    /// The underlying transport server.
    pub server: Arc<dyn MicroserviceServer>,
}

impl EventServer {
    /// Creates a [`ProviderDefinition`] that injects [`TransportConfig`].
    ///
    /// Register with `#[module(providers = [EventServer, ...])]`.
    pub fn provider_definition() -> ProviderDefinition {
        ProviderDefinition::factory::<Self, _, _>(
            Scope::Singleton,
            vec![Dependency::required::<TransportConfig>()],
            |resolver| async move {
                let config: Arc<TransportConfig> = resolver.resolve().await?;
                let server = create_transport_server(&config);
                Ok(Self { server })
            },
        )
        .eager()
    }

    /// Creates a paired [`EventClient`] and [`EventServer`] over in-memory transport.
    ///
    /// Both endpoints share the same transport channel, so events emitted by
    /// the client are received by the server. Useful for single-process
    /// applications and tests.
    ///
    /// # Example (requires `microservices` feature)
    ///
    /// ```ignore
    /// use ironic::distributed::transport_provider::EventServer;
    /// use std::sync::Arc;
    ///
    /// let (client, server) = EventServer::paired(16);
    /// server.on_event("ping", Arc::new(|_, _| Box::pin(async { Ok(()) })));
    /// ```
    #[must_use]
    pub fn paired(capacity: usize) -> (EventClient, Self) {
        let (client, server) = InMemoryServer::pair(capacity);
        let event_client = EventClient {
            client: TransportClient::InMemory(client),
        };
        let event_server = Self {
            server: Arc::new(server),
        };
        (event_client, event_server)
    }
}

impl MicroserviceServer for EventServer {
    fn listen(&self) -> ServerFuture<()> {
        self.server.listen()
    }

    fn on_message(&self, pattern: &str, handler: MessageHandler) {
        self.server.on_message(pattern, handler);
    }

    fn on_event(&self, pattern: &str, handler: EventHandler) {
        self.server.on_event(pattern, handler);
    }

    fn close(&self) -> ServerFuture<()> {
        self.server.close()
    }
}

impl OnApplicationBootstrap for EventServer {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.server
                .listen()
                .await
                .map_err(|e| LifecycleError::new(format!("EVENT_SERVER_LISTEN: {}", e.0)))?;
            tracing::info!("event server listening");
            Ok(())
        })
    }
}

impl OnApplicationShutdown for EventServer {
    fn on_application_shutdown(&self, _signal: ShutdownSignal) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.server.close().await.ok();
            tracing::info!("event server disconnected");
            Ok(())
        })
    }
}
