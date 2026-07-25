#![allow(
    clippy::type_complexity,
    clippy::collapsible_if,
    clippy::while_let_loop
)]
//! Live Redis pub/sub transport backend implementing [`MicroserviceClient`] and
//! [`MicroserviceServer`].
//!
//! Requires the `transport-redis` feature and a running Redis instance.

use std::{collections::HashMap, sync::Arc};

use redis::aio::MultiplexedConnection;

use crate::distributed::microservices::{
    ClientFuture, EventHandler, MessageContext, MessageHandler, MicroserviceClient,
    MicroserviceServer, ServerFuture, TransportError, generate_correlation_id,
};

// ---------------------------------------------------------------------------
// Helper: run a Redis command on a connection
// ---------------------------------------------------------------------------

async fn redis_publish(
    conn: &mut MultiplexedConnection,
    channel: &str,
    data: &[u8],
) -> Result<(), TransportError> {
    redis::cmd("PUBLISH")
        .arg(channel)
        .arg(data)
        .query_async(conn)
        .await
        .map_err(|e| TransportError(e.to_string()))
}

// ---------------------------------------------------------------------------
// Redis Client
// ---------------------------------------------------------------------------

/// A Redis pub/sub microservice client.
pub struct RedisClient {
    config: RedisClientConfig,
    pub_conn: Arc<tokio::sync::OnceCell<MultiplexedConnection>>,
    response_handlers: Arc<
        tokio::sync::Mutex<
            HashMap<String, tokio::sync::oneshot::Sender<Result<Vec<u8>, TransportError>>>,
        >,
    >,
}

/// Configuration for a Redis microservice client.
#[derive(Clone, Debug)]
pub struct RedisClientConfig {
    /// Redis connection URL (e.g., `redis://127.0.0.1:6379`).
    pub url: String,
    /// Number of retry attempts for reconnection.
    pub retry_attempts: usize,
    /// Delay between retry attempts in milliseconds.
    pub retry_delay_ms: u64,
}

impl Default for RedisClientConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".into(),
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl RedisClient {
    /// Creates a new Redis client with the given config.
    #[must_use]
    pub fn new(config: RedisClientConfig) -> Self {
        Self {
            config,
            pub_conn: Arc::new(tokio::sync::OnceCell::new()),
            response_handlers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceClient for RedisClient {
    fn connect(&self) -> ClientFuture<()> {
        let config = self.config.clone();
        let pub_conn = Arc::clone(&self.pub_conn);
        let response_handlers = Arc::clone(&self.response_handlers);

        Box::pin(async move {
            let client = redis::Client::open(config.url.as_str())
                .map_err(|e| TransportError(e.to_string()))?;
            let conn = client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            // Create pubsub for listening to replies
            let sub_client = redis::Client::open(config.url.as_str())
                .map_err(|e| TransportError(e.to_string()))?;
            let mut pubsub = sub_client
                .get_async_pubsub()
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            pubsub
                .psubscribe("*.reply")
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let handlers = Arc::clone(&response_handlers);
            tokio::spawn(async move {
                let mut msg_stream = pubsub.into_on_message();
                loop {
                    match futures_util::StreamExt::next(&mut msg_stream).await {
                        Some(msg) => {
                            let channel: String = msg.get_channel_name().to_string();
                            let payload: Vec<u8> = msg.get_payload().unwrap_or_default();
                            if let Some(cid) = channel.strip_suffix(".reply") {
                                let mut map = handlers.lock().await;
                                if let Some(tx) = map.remove(cid) {
                                    let _ = tx.send(Ok(payload));
                                }
                            }
                        }
                        None => break,
                    }
                }
            });

            pub_conn
                .set(conn)
                .map_err(|_| TransportError("already connected".into()))?;
            Ok(())
        })
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        let pub_conn = Arc::clone(&self.pub_conn);
        let handlers = Arc::clone(&self.response_handlers);
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let conn = pub_conn
                .get()
                .ok_or_else(|| TransportError("Redis client not connected".into()))?;
            let correlation_id = generate_correlation_id();

            let (tx, rx) = tokio::sync::oneshot::channel();
            handlers.lock().await.insert(correlation_id.clone(), tx);

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let envelope = serde_json::to_vec(&serde_json::json!({
                "correlation_id": correlation_id,
                "data": data_str,
            }))
            .map_err(|e| TransportError(e.to_string()))?;

            let mut conn_clone = conn.clone();
            redis_publish(&mut conn_clone, &pattern, &envelope).await?;

            let response = rx
                .await
                .map_err(|_| TransportError("response channel closed".into()))??;

            serde_json::from_slice(&response).map_err(|e| TransportError(e.to_string()))
        })
    }

    fn emit<T>(&self, pattern: &str, data: &T) -> ClientFuture<()>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
    {
        let pub_conn = Arc::clone(&self.pub_conn);
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let conn = pub_conn
                .get()
                .ok_or_else(|| TransportError("Redis client not connected".into()))?;

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let envelope = serde_json::to_vec(&serde_json::json!({
                "correlation_id": generate_correlation_id(),
                "data": data_str,
            }))
            .map_err(|e| TransportError(e.to_string()))?;

            let mut conn_clone = conn.clone();
            redis_publish(&mut conn_clone, &pattern, &envelope).await
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// Redis Server
// ---------------------------------------------------------------------------

/// A Redis pub/sub microservice server.
pub struct RedisServer {
    config: RedisServerConfig,
    handlers: Arc<std::sync::Mutex<HashMap<String, MessageHandler>>>,
    event_handlers: Arc<std::sync::Mutex<HashMap<String, EventHandler>>>,
}

/// Configuration for a Redis microservice server.
#[derive(Clone, Debug)]
pub struct RedisServerConfig {
    /// Redis connection URL.
    pub url: String,
    /// Whether to use Redis psubscribe/pmessage for wildcard patterns.
    pub wildcards: bool,
    /// Number of retry attempts.
    pub retry_attempts: usize,
    /// Delay between retry attempts in milliseconds.
    pub retry_delay_ms: u64,
}

impl Default for RedisServerConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".into(),
            wildcards: false,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

impl RedisServer {
    /// Creates a new Redis server with the given config.
    #[must_use]
    pub fn new(config: RedisServerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
            event_handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceServer for RedisServer {
    fn listen(&self) -> ServerFuture<()> {
        let config = self.config.clone();
        let handlers = Arc::clone(&self.handlers);
        let event_handlers = Arc::clone(&self.event_handlers);

        Box::pin(async move {
            let client = redis::Client::open(config.url.as_str())
                .map_err(|e| TransportError(e.to_string()))?;
            let mut pubsub = client
                .get_async_pubsub()
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            // Separate client for publishing replies
            let reply_client = redis::Client::open(config.url.as_str())
                .map_err(|e| TransportError(e.to_string()))?;
            let reply_conn = reply_client
                .get_multiplexed_async_connection()
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            // Collect and subscribe to all registered patterns
            let registered_patterns: Vec<String> = {
                let h = handlers.lock().unwrap();
                h.keys().cloned().collect()
            };
            let event_patterns: Vec<String> = {
                let h = event_handlers.lock().unwrap();
                h.keys().cloned().collect()
            };

            for pattern in &registered_patterns {
                pubsub
                    .subscribe(pattern.as_str())
                    .await
                    .map_err(|e| TransportError(e.to_string()))?;
            }
            for pattern in &event_patterns {
                pubsub
                    .subscribe(pattern.as_str())
                    .await
                    .map_err(|e| TransportError(e.to_string()))?;
            }

            let handlers_clone = Arc::clone(&handlers);
            let event_handlers_clone = Arc::clone(&event_handlers);

            let mut msg_stream = pubsub.into_on_message();
            tokio::spawn(async move {
                loop {
                    match futures_util::StreamExt::next(&mut msg_stream).await {
                        Some(msg) => {
                            let channel: String = msg.get_channel_name().to_string();
                            let payload: Vec<u8> = msg.get_payload().unwrap_or_default();

                            if let Ok(parsed) =
                                serde_json::from_slice::<serde_json::Value>(&payload)
                            {
                                let correlation_id =
                                    parsed["correlation_id"].as_str().unwrap_or("").to_string();
                                let data: Vec<u8> = parsed["data"]
                                    .as_str()
                                    .map(|s| s.as_bytes().to_vec())
                                    .unwrap_or_default();

                                let context = MessageContext {
                                    pattern: channel.clone(),
                                    correlation_id,
                                    headers: std::collections::BTreeMap::new(),
                                };

                                // Check message handlers (request-response)
                                let handler_opt = {
                                    let h = handlers_clone.lock().unwrap();
                                    h.get(&channel).cloned()
                                };
                                if let Some(handler) = handler_opt {
                                    let result = handler(data, context).await;
                                    if let Ok(response) = result {
                                        let reply_channel = format!("{channel}.reply");
                                        let mut rc = reply_conn.clone();
                                        let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
                                            .arg(&reply_channel)
                                            .arg(&response)
                                            .query_async(&mut rc)
                                            .await;
                                    }
                                } else {
                                    // Check event handlers (fire-and-forget)
                                    let handler_opt = {
                                        let h = event_handlers_clone.lock().unwrap();
                                        h.get(&channel).cloned()
                                    };
                                    if let Some(handler) = handler_opt {
                                        let _ = handler(data, context).await;
                                    }
                                }
                            }
                        }
                        None => break,
                    }
                }
            });

            Ok(())
        })
    }

    fn on_message(&self, pattern: &str, handler: MessageHandler) {
        self.handlers
            .lock()
            .unwrap()
            .insert(pattern.to_string(), handler);
    }

    fn on_event(&self, pattern: &str, handler: EventHandler) {
        self.event_handlers
            .lock()
            .unwrap()
            .insert(pattern.to_string(), handler);
    }

    fn close(&self) -> ServerFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}
