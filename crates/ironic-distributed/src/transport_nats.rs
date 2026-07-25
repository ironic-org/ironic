#![allow(
    deprecated,
    clippy::type_complexity,
    clippy::too_many_lines,
    clippy::collapsible_if
)]

//! Live NATS transport backend using the `async-nats` crate.
//! Requires the `transport-nats` feature and a running NATS server.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, OnceCell};

use crate::distributed::microservices::{
    ClientFuture, EventHandler, MessageContext, MessageHandler, MicroserviceClient,
    MicroserviceServer, ServerFuture, TransportError, generate_correlation_id,
};

/// A live NATS microservice client.
pub struct NatsClient {
    client: Arc<OnceCell<async_nats::Client>>,
    config: NatsClientConfig,
    response_handlers:
        Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<Result<Vec<u8>, TransportError>>>>>,
}

/// Configuration for a NATS microservice client.
#[derive(Clone, Debug)]
pub struct NatsClientConfig {
    /// NATS server URL.
    pub url: String,
    /// Subject prefix.
    pub subject_prefix: String,
}

impl Default for NatsClientConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".into(),
            subject_prefix: "ironic".into(),
        }
    }
}

impl NatsClient {
    /// Creates a new NATS client.
    #[must_use]
    pub fn new(config: NatsClientConfig) -> Self {
        Self {
            client: Arc::new(OnceCell::new()),
            config,
            response_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceClient for NatsClient {
    fn connect(&self) -> ClientFuture<()> {
        let config = self.config.clone();
        let client_cell = Arc::clone(&self.client);
        let handlers = Arc::clone(&self.response_handlers);

        Box::pin(async move {
            let client = async_nats::connect(&config.url)
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let handlers_clone = Arc::clone(&handlers);
            let prefix = config.subject_prefix.clone();
            let sub = client
                .subscribe(format!("{prefix}.reply.>"))
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            tokio::spawn(async move {
                let mut sub = sub;
                while let Some(msg) = futures_util::StreamExt::next(&mut sub).await {
                    let subject = msg.subject;
                    if let Some(cid) = subject.strip_prefix(&format!("{prefix}.reply.")) {
                        if let Ok(parsed) =
                            serde_json::from_slice::<serde_json::Value>(&msg.payload)
                        {
                            let data: Vec<u8> = parsed["data"]
                                .as_str()
                                .map(|s| s.as_bytes().to_vec())
                                .unwrap_or_default();
                            let mut map = handlers_clone.lock().await;
                            if let Some(tx) = map.remove(cid) {
                                let _ = tx.send(Ok(data));
                            }
                        }
                    }
                }
            });

            client_cell
                .set(client)
                .map_err(|_| TransportError("already connected".into()))?;
            Ok(())
        })
    }

    fn send<T, R>(&self, pattern: &str, data: &T) -> ClientFuture<R>
    where
        T: serde::Serialize + Send + Sync + ?Sized,
        R: serde::de::DeserializeOwned + Send,
    {
        let client_cell = Arc::clone(&self.client);
        let handlers = Arc::clone(&self.response_handlers);
        let prefix = self.config.subject_prefix.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let client = client_cell
                .get()
                .ok_or_else(|| TransportError("NATS client not connected".into()))?;
            let correlation_id = generate_correlation_id();
            let (tx, rx) = tokio::sync::oneshot::channel();
            handlers.lock().await.insert(correlation_id.clone(), tx);

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let envelope = serde_json::json!({
                "correlation_id": correlation_id,
                "data": data_str,
            });
            let body = serde_json::to_vec(&envelope).map_err(|e| TransportError(e.to_string()))?;

            client
                .publish(format!("{prefix}.{pattern}"), body.into())
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
        let client_cell = Arc::clone(&self.client);
        let prefix = self.config.subject_prefix.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let client = client_cell
                .get()
                .ok_or_else(|| TransportError("NATS client not connected".into()))?;

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let envelope = serde_json::json!({
                "correlation_id": generate_correlation_id(),
                "data": data_str,
            });
            let body = serde_json::to_vec(&envelope).map_err(|e| TransportError(e.to_string()))?;

            client
                .publish(format!("{prefix}.{pattern}"), body.into())
                .await
                .map_err(|e| TransportError(e.to_string()))
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

/// A live NATS microservice server.
pub struct NatsServer {
    config: NatsServerConfig,
    handlers: Arc<std::sync::Mutex<HashMap<String, MessageHandler>>>,
    event_handlers: Arc<std::sync::Mutex<HashMap<String, EventHandler>>>,
}

/// Configuration for a NATS microservice server.
#[derive(Clone, Debug)]
pub struct NatsServerConfig {
    /// NATS server URL.
    pub url: String,
    /// Subject prefix.
    pub subject_prefix: String,
}

impl Default for NatsServerConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".into(),
            subject_prefix: "ironic".into(),
        }
    }
}

impl NatsServer {
    /// Creates a new NATS server.
    #[must_use]
    pub fn new(config: NatsServerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
            event_handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceServer for NatsServer {
    fn listen(&self) -> ServerFuture<()> {
        let config = self.config.clone();
        let handlers = Arc::clone(&self.handlers);
        let event_handlers = Arc::clone(&self.event_handlers);

        Box::pin(async move {
            let client = async_nats::connect(&config.url)
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

            let prefix = config.subject_prefix.clone();
            let handlers_c = Arc::clone(&handlers);
            let event_handlers_c = Arc::clone(&event_handlers);

            for pattern in &registered_patterns {
                let sub = client
                    .subscribe(format!("{prefix}.{pattern}"))
                    .await
                    .map_err(|e| TransportError(e.to_string()))?;
                let h = Arc::clone(&handlers_c);
                let eh = Arc::clone(&event_handlers_c);
                let p = prefix.clone();
                let client_pub = client.clone();
                tokio::spawn(async move {
                    let mut sub = sub;
                    while let Some(msg) = futures_util::StreamExt::next(&mut sub).await {
                        let pattern = msg
                            .subject
                            .strip_prefix(&format!("{p}."))
                            .unwrap_or(&msg.subject)
                            .to_string();
                        if let Ok(parsed) =
                            serde_json::from_slice::<serde_json::Value>(&msg.payload)
                        {
                            let correlation_id =
                                parsed["correlation_id"].as_str().unwrap_or("").to_string();
                            let data: Vec<u8> = parsed["data"]
                                .as_str()
                                .map(|s| s.as_bytes().to_vec())
                                .unwrap_or_default();
                            let cid = correlation_id.clone();
                            let context = MessageContext {
                                pattern: pattern.clone(),
                                correlation_id,
                                headers: std::collections::BTreeMap::new(),
                            };
                            let handler_opt = {
                                let g = h.lock().unwrap();
                                g.get(&pattern).cloned()
                            };
                            if let Some(handler) = handler_opt {
                                let result = handler(data, context).await;
                                if let Ok(response) = result {
                                    let _ = client_pub
                                        .publish(format!("{p}.reply.{cid}"), response.into())
                                        .await;
                                }
                            } else {
                                let handler_opt = {
                                    let g = eh.lock().unwrap();
                                    g.get(&pattern).cloned()
                                };
                                if let Some(handler) = handler_opt {
                                    let _ = handler(data, context).await;
                                }
                            }
                        }
                    }
                });
            }
            for pattern in &event_patterns {
                let sub = client
                    .subscribe(format!("{prefix}.{pattern}"))
                    .await
                    .map_err(|e| TransportError(e.to_string()))?;
                let eh = Arc::clone(&event_handlers_c);
                let p = prefix.clone();
                tokio::spawn(async move {
                    let mut sub = sub;
                    while let Some(msg) = futures_util::StreamExt::next(&mut sub).await {
                        let pattern = msg
                            .subject
                            .strip_prefix(&format!("{p}."))
                            .unwrap_or(&msg.subject)
                            .to_string();
                        if let Ok(parsed) =
                            serde_json::from_slice::<serde_json::Value>(&msg.payload)
                        {
                            let data: Vec<u8> = parsed["data"]
                                .as_str()
                                .map(|s| s.as_bytes().to_vec())
                                .unwrap_or_default();
                            let context = MessageContext {
                                pattern,
                                correlation_id: String::new(),
                                headers: std::collections::BTreeMap::new(),
                            };
                            let handler_opt = {
                                let g = eh.lock().unwrap();
                                g.get(&context.pattern).cloned()
                            };
                            if let Some(handler) = handler_opt {
                                let _ = handler(data, context).await;
                            }
                        }
                    }
                });
            }

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
