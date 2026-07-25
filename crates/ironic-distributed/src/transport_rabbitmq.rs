#![allow(
    clippy::type_complexity,
    clippy::collapsible_if,
    clippy::while_let_loop
)]
//! Live RabbitMQ transport backend implementing [`MicroserviceClient`] and
//! [`MicroserviceServer`] using the `lapin` crate.

use std::{collections::HashMap, sync::Arc};

use futures_util::StreamExt;
use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{
        BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    types::ShortString,
};

use crate::distributed::microservices::{
    ClientFuture, EventHandler, MessageContext, MessageHandler, MicroserviceClient,
    MicroserviceServer, ServerFuture, TransportError, generate_correlation_id,
};

fn ss(s: &str) -> ShortString {
    ShortString::from(s)
}

// ---------------------------------------------------------------------------
// RabbitMQ Client
// ---------------------------------------------------------------------------

/// A RabbitMQ microservice client.
pub struct RmqClient {
    config: RmqClientConfig,
    channel: Arc<tokio::sync::OnceCell<Channel>>,
    response_handlers: Arc<
        tokio::sync::Mutex<
            HashMap<String, tokio::sync::oneshot::Sender<Result<Vec<u8>, TransportError>>>,
        >,
    >,
}

/// Configuration for a RabbitMQ microservice client.
#[derive(Clone, Debug)]
pub struct RmqClientConfig {
    /// AMQP connection URL.
    pub url: String,
    /// Exchange name.
    pub exchange: String,
}

impl Default for RmqClientConfig {
    fn default() -> Self {
        Self {
            url: "amqp://guest:guest@127.0.0.1:5672".into(),
            exchange: "ironic".into(),
        }
    }
}

impl RmqClient {
    /// Creates a new RabbitMQ client.
    #[must_use]
    pub fn new(config: RmqClientConfig) -> Self {
        Self {
            config,
            channel: Arc::new(tokio::sync::OnceCell::new()),
            response_handlers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceClient for RmqClient {
    fn connect(&self) -> ClientFuture<()> {
        let config = self.config.clone();
        let channel = Arc::clone(&self.channel);
        let response_handlers = Arc::clone(&self.response_handlers);

        Box::pin(async move {
            let conn = Connection::connect(&config.url, ConnectionProperties::default())
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            let ch = conn
                .create_channel()
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            ch.exchange_declare(
                ss(&config.exchange),
                ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| TransportError(e.to_string()))?;

            let reply_queue = ch
                .queue_declare(
                    ss(""),
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            let reply_queue_name = reply_queue.name().as_str().to_string();

            ch.queue_bind(
                ss(&reply_queue_name),
                ss(&config.exchange),
                ss("*.reply"),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| TransportError(e.to_string()))?;

            let consumer = ch
                .basic_consume(
                    ss(&reply_queue_name),
                    ss(""),
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let handlers = Arc::clone(&response_handlers);
            tokio::spawn(async move {
                let mut stream = consumer;
                while let Some(Ok(delivery)) = stream.next().await {
                    let cid = delivery
                        .properties
                        .correlation_id()
                        .as_ref()
                        .map(|s| s.as_str().to_string());
                    if let Some(cid) = cid {
                        let mut map = handlers.lock().await;
                        if let Some(tx) = map.remove(&cid) {
                            let payload = delivery.data.to_vec();
                            let _ = tx.send(Ok(payload));
                        }
                    }
                    let _ = delivery
                        .acker
                        .ack(lapin::options::BasicAckOptions::default())
                        .await;
                }
            });

            channel
                .set(ch)
                .map_err(|_| TransportError("already connected".into()))?;
            Ok(())
        })
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        let channel = Arc::clone(&self.channel);
        let handlers = Arc::clone(&self.response_handlers);
        let exchange = self.config.exchange.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let ch = channel
                .get()
                .ok_or_else(|| TransportError("RabbitMQ client not connected".into()))?;
            let correlation_id = generate_correlation_id();

            let (tx, rx) = tokio::sync::oneshot::channel();
            handlers.lock().await.insert(correlation_id.clone(), tx);

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let body = serde_json::to_vec(&serde_json::json!({
                "correlation_id": correlation_id,
                "data": data_str,
            }))
            .map_err(|e| TransportError(e.to_string()))?;

            ch.basic_publish(
                ss(&exchange),
                ss(&pattern),
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default()
                    .with_correlation_id(ss(&correlation_id))
                    .with_reply_to(ss("amq.rabbitmq.reply-to")),
            )
            .await
            .map_err(|e| TransportError(e.to_string()))?;

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
        let channel = Arc::clone(&self.channel);
        let exchange = self.config.exchange.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let ch = channel
                .get()
                .ok_or_else(|| TransportError("RabbitMQ client not connected".into()))?;

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let body = serde_json::to_vec(&serde_json::json!({
                "correlation_id": generate_correlation_id(),
                "data": data_str,
            }))
            .map_err(|e| TransportError(e.to_string()))?;

            ch.basic_publish(
                ss(&exchange),
                ss(&pattern),
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default(),
            )
            .await
            .map_err(|e| TransportError(e.to_string()))?;
            Ok(())
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// RabbitMQ Server
// ---------------------------------------------------------------------------

/// A RabbitMQ microservice server.
pub struct RmqServer {
    config: RmqServerConfig,
    handlers: Arc<std::sync::Mutex<HashMap<String, MessageHandler>>>,
    event_handlers: Arc<std::sync::Mutex<HashMap<String, EventHandler>>>,
}

/// Configuration for a RabbitMQ microservice server.
#[derive(Clone, Debug)]
pub struct RmqServerConfig {
    /// AMQP connection URL.
    pub url: String,
    /// Exchange name.
    pub exchange: String,
    /// Queue name (empty for auto-generated).
    pub queue: String,
}

impl Default for RmqServerConfig {
    fn default() -> Self {
        Self {
            url: "amqp://guest:guest@127.0.0.1:5672".into(),
            exchange: "ironic".into(),
            queue: String::new(),
        }
    }
}

impl RmqServer {
    /// Creates a new RabbitMQ server.
    #[must_use]
    pub fn new(config: RmqServerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
            event_handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceServer for RmqServer {
    fn listen(&self) -> ServerFuture<()> {
        let config = self.config.clone();
        let handlers = Arc::clone(&self.handlers);
        let event_handlers = Arc::clone(&self.event_handlers);

        Box::pin(async move {
            let conn = Connection::connect(&config.url, ConnectionProperties::default())
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            let ch = conn
                .create_channel()
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            ch.exchange_declare(
                ss(&config.exchange),
                ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| TransportError(e.to_string()))?;

            let registered_patterns: Vec<String> = {
                let h = handlers.lock().unwrap();
                h.keys().cloned().collect()
            };
            let event_patterns: Vec<String> = {
                let h = event_handlers.lock().unwrap();
                h.keys().cloned().collect()
            };

            let queue_name = if config.queue.is_empty() {
                let q = ch
                    .queue_declare(
                        ss(""),
                        QueueDeclareOptions::default(),
                        FieldTable::default(),
                    )
                    .await
                    .map_err(|e| TransportError(e.to_string()))?;
                q.name().as_str().to_string()
            } else {
                ch.queue_declare(
                    ss(&config.queue),
                    QueueDeclareOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| TransportError(e.to_string()))?;
                config.queue.clone()
            };

            for pattern in &registered_patterns {
                ch.queue_bind(
                    ss(&queue_name),
                    ss(&config.exchange),
                    ss(pattern),
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            }
            for pattern in &event_patterns {
                ch.queue_bind(
                    ss(&queue_name),
                    ss(&config.exchange),
                    ss(pattern),
                    QueueBindOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| TransportError(e.to_string()))?;
            }

            let reply_ch = conn
                .create_channel()
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let consumer = ch
                .basic_consume(
                    ss(&queue_name),
                    ss(""),
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let handlers_clone = Arc::clone(&handlers);
            let event_handlers_clone = Arc::clone(&event_handlers);
            let _exchange = config.exchange.clone();

            tokio::spawn(async move {
                let mut stream = consumer;
                while let Some(Ok(delivery)) = stream.next().await {
                    let channel = delivery.routing_key.as_str().to_string();
                    let payload = delivery.data.to_vec();

                    if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&payload) {
                        let correlation_id =
                            parsed["correlation_id"].as_str().unwrap_or("").to_string();
                        let data: Vec<u8> = parsed["data"]
                            .as_str()
                            .map(|s| s.as_bytes().to_vec())
                            .unwrap_or_default();
                        let reply_to = delivery
                            .properties
                            .reply_to()
                            .as_ref()
                            .map(|s| s.as_str().to_string());

                        let context = MessageContext {
                            pattern: channel.clone(),
                            correlation_id,
                            headers: std::collections::BTreeMap::new(),
                        };

                        let handler_opt = {
                            let h = handlers_clone.lock().unwrap();
                            h.get(&channel).cloned()
                        };
                        if let Some(handler) = handler_opt {
                            let result = handler(data, context).await;
                            if let Ok(response) = result {
                                let _ = reply_ch
                                    .basic_publish(
                                        ss(""),
                                        ss(&reply_to.unwrap_or_default()),
                                        BasicPublishOptions::default(),
                                        &response,
                                        BasicProperties::default(),
                                    )
                                    .await;
                            }
                        } else {
                            let handler_opt = {
                                let h = event_handlers_clone.lock().unwrap();
                                h.get(&channel).cloned()
                            };
                            if let Some(handler) = handler_opt {
                                let _ = handler(data, context).await;
                            }
                        }
                    }
                    let _ = delivery
                        .acker
                        .ack(lapin::options::BasicAckOptions::default())
                        .await;
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
