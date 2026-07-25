#![allow(
    clippy::type_complexity,
    clippy::collapsible_if,
    clippy::while_let_loop,
    clippy::useless_conversion
)]
//! Kafka transport backend implementing [`MicroserviceClient`] and
//! [`MicroserviceServer`] using the `kafka` crate.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::distributed::microservices::{
    ClientFuture, EventHandler, MessageContext, MessageHandler, MicroserviceClient,
    MicroserviceServer, ServerFuture, TransportError, generate_correlation_id,
};

// ---------------------------------------------------------------------------
// Kafka Client
// ---------------------------------------------------------------------------

/// A Kafka microservice client (producer + reply consumer).
pub struct KafkaClient {
    config: KafkaClientConfig,
    producer: Arc<Mutex<Option<kafka::producer::Producer>>>,
    response_handlers: Arc<
        tokio::sync::Mutex<
            HashMap<String, tokio::sync::oneshot::Sender<Result<Vec<u8>, TransportError>>>,
        >,
    >,
}

/// Configuration for a Kafka microservice client.
#[derive(Clone, Debug)]
pub struct KafkaClientConfig {
    /// Comma-separated list of bootstrap brokers.
    pub brokers: String,
    /// Topic to produce to.
    pub topic: String,
}

impl Default for KafkaClientConfig {
    fn default() -> Self {
        Self {
            brokers: "127.0.0.1:9092".into(),
            topic: "ironic".into(),
        }
    }
}

impl KafkaClient {
    /// Creates a new Kafka client.
    #[must_use]
    pub fn new(config: KafkaClientConfig) -> Self {
        Self {
            config,
            producer: Arc::new(Mutex::new(None)),
            response_handlers: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceClient for KafkaClient {
    fn connect(&self) -> ClientFuture<()> {
        let config = self.config.clone();
        let producer_cell = Arc::clone(&self.producer);
        let handlers = Arc::clone(&self.response_handlers);

        Box::pin(async move {
            let hosts: Vec<String> = config
                .brokers
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let brokers = hosts.clone();
            let producer = tokio::task::spawn_blocking(move || {
                kafka::producer::Producer::from_hosts(brokers)
                    .create()
                    .map_err(|e| TransportError(e.to_string()))
            })
            .await
            .map_err(|e| TransportError(e.to_string()))??;

            *producer_cell.lock().await = Some(producer);

            // Reply consumer polling
            let reply_topic = format!("{}_reply", config.topic);
            let handlers_clone = Arc::clone(&handlers);
            let hosts_clone = hosts.clone();
            tokio::spawn(async move {
                loop {
                    let hosts = hosts_clone.clone();
                    let topic = reply_topic.clone();
                    let h = Arc::clone(&handlers_clone);
                    let _ = tokio::task::spawn_blocking(move || {
                        let mut consumer = match kafka::consumer::Consumer::from_hosts(hosts)
                            .with_topic(topic)
                            .with_fallback_offset(kafka::consumer::FetchOffset::Earliest)
                            .create()
                        {
                            Ok(c) => c,
                            Err(_) => return,
                        };
                        loop {
                            if let Ok(message_sets) = consumer.poll() {
                                for ms in message_sets.iter() {
                                    for msg in ms.messages() {
                                        let val = msg.value;
                                        if !val.is_empty() {
                                            if let Ok(parsed) =
                                                serde_json::from_slice::<serde_json::Value>(val)
                                            {
                                                let cid = parsed["correlation_id"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string();
                                                let data = parsed["data"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .as_bytes()
                                                    .to_vec();
                                                // Use block_on to send through oneshot
                                                if let Ok(mut map) = h.try_lock() {
                                                    if let Some(tx) = map.remove(&cid) {
                                                        let _ = tx.send(Ok(data));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })
                    .await;
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });

            Ok(())
        })
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        let producer_cell = Arc::clone(&self.producer);
        let handlers = Arc::clone(&self.response_handlers);
        let topic = self.config.topic.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let mut prod = producer_cell.lock().await;
            let producer = prod
                .as_mut()
                .ok_or_else(|| TransportError("Kafka client not connected".into()))?;
            let correlation_id = generate_correlation_id();

            let (tx, rx) = tokio::sync::oneshot::channel();
            handlers.lock().await.insert(correlation_id.clone(), tx);

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let body = serde_json::json!({
                "correlation_id": correlation_id,
                "data": data_str,
            });
            let body_bytes =
                serde_json::to_vec(&body).map_err(|e| TransportError(e.to_string()))?;

            let rec =
                kafka::producer::Record::from_key_value(&topic, pattern.as_str(), &body_bytes[..]);
            producer
                .send(&rec)
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
        let producer_cell = Arc::clone(&self.producer);
        let topic = self.config.topic.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let mut prod = producer_cell.lock().await;
            let producer = prod
                .as_mut()
                .ok_or_else(|| TransportError("Kafka client not connected".into()))?;

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let body = serde_json::json!({
                "correlation_id": generate_correlation_id(),
                "data": data_str,
            });
            let body_bytes =
                serde_json::to_vec(&body).map_err(|e| TransportError(e.to_string()))?;

            let rec =
                kafka::producer::Record::from_key_value(&topic, pattern.as_str(), &body_bytes[..]);
            producer
                .send(&rec)
                .map_err(|e| TransportError(e.to_string()))?;

            Ok(())
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

// ---------------------------------------------------------------------------
// Kafka Server
// ---------------------------------------------------------------------------

/// A Kafka microservice server (consumer + reply producer).
pub struct KafkaServer {
    config: KafkaServerConfig,
    handlers: Arc<std::sync::Mutex<HashMap<String, MessageHandler>>>,
    event_handlers: Arc<std::sync::Mutex<HashMap<String, EventHandler>>>,
}

/// Configuration for a Kafka microservice server.
#[derive(Clone, Debug)]
pub struct KafkaServerConfig {
    /// Comma-separated list of bootstrap brokers.
    pub brokers: String,
    /// Topic to consume from.
    pub topic: String,
    /// Consumer group ID.
    pub group_id: String,
}

impl Default for KafkaServerConfig {
    fn default() -> Self {
        Self {
            brokers: "127.0.0.1:9092".into(),
            topic: "ironic".into(),
            group_id: "ironic-server".into(),
        }
    }
}

impl KafkaServer {
    /// Creates a new Kafka server.
    #[must_use]
    pub fn new(config: KafkaServerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
            event_handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceServer for KafkaServer {
    fn listen(&self) -> ServerFuture<()> {
        let config = self.config.clone();
        let handlers = Arc::clone(&self.handlers);
        let event_handlers = Arc::clone(&self.event_handlers);

        Box::pin(async move {
            let hosts: Vec<String> = config
                .brokers
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            let topic = config.topic.clone();
            let reply_topic = format!("{topic}_reply");

            // Create a producer for sending replies
            let prod_hosts = hosts.clone();
            let _prod = tokio::task::spawn_blocking(move || {
                kafka::producer::Producer::from_hosts(prod_hosts)
                    .create()
                    .map_err(|e| TransportError(e.to_string()))
            })
            .await
            .map_err(|e| TransportError(e.to_string()))?;

            let handlers_clone = Arc::clone(&handlers);
            let event_handlers_clone = Arc::clone(&event_handlers);
            let reply_topic_clone = reply_topic.clone();
            let hosts_clone = hosts.clone();

            tokio::spawn(async move {
                loop {
                    let hosts = hosts_clone.clone();
                    let topic = topic.clone();
                    let reply_topic = reply_topic_clone.clone();
                    let h = Arc::clone(&handlers_clone);
                    let eh = Arc::clone(&event_handlers_clone);

                    let _ = tokio::task::spawn_blocking(move || {
                        let mut consumer = match kafka::consumer::Consumer::from_hosts(hosts)
                            .with_topic(topic)
                            .with_fallback_offset(kafka::consumer::FetchOffset::Earliest)
                            .create()
                        {
                            Ok(c) => c,
                            Err(_) => return,
                        };
                        loop {
                            if let Ok(message_sets) = consumer.poll() {
                                for ms in message_sets.iter() {
                                    for msg in ms.messages() {
                                        let key = String::from_utf8_lossy(msg.key).to_string();
                                        let val = msg.value;
                                        if !val.is_empty() {
                                            if let Ok(parsed) =
                                                serde_json::from_slice::<serde_json::Value>(val)
                                            {
                                                let correlation_id = parsed["correlation_id"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string();
                                                let data: Vec<u8> = parsed["data"]
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .as_bytes()
                                                    .to_vec();

                                                // Process synchronously in blocking context
                                                let cid = correlation_id.clone();
                                                let context = MessageContext {
                                                    pattern: key.clone(),
                                                    correlation_id,
                                                    headers: std::collections::BTreeMap::new(),
                                                };

                                                let handler_opt = {
                                                    let guard = h.lock().unwrap();
                                                    guard.get(&key).cloned()
                                                };
                                                if let Some(handler) = handler_opt {
                                                    let result = tokio::runtime::Handle::current()
                                                        .block_on(handler(data, context));
                                                    if let Ok(response) = result {
                                                        let rec =
                                                            kafka::producer::Record::from_key_value(
                                                                &reply_topic,
                                                                cid.as_str(),
                                                                &response[..],
                                                            );
                                                        if let Ok(mut prod) =
                                                            kafka::producer::Producer::from_hosts(
                                                                Vec::new(),
                                                            )
                                                            .create()
                                                        {
                                                            let _ = prod.send(&rec);
                                                        }
                                                    }
                                                } else {
                                                    let handler_opt = {
                                                        let guard = eh.lock().unwrap();
                                                        guard.get(&key).cloned()
                                                    };
                                                    if let Some(handler) = handler_opt {
                                                        let _ = tokio::runtime::Handle::current()
                                                            .block_on(handler(data, context));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })
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
