#![allow(deprecated)]

//! Live MQTT transport backend using the `rumqttc` crate.
//! Requires the `transport-mqtt` feature and a running MQTT broker.

use std::{collections::HashMap, sync::Arc};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::sync::{Mutex, OnceCell};

use crate::distributed::microservices::{
    ClientFuture, Envelope, EventHandler, MessageContext, MessageHandler, MicroserviceClient,
    MicroserviceServer, ServerFuture, TransportError, generate_correlation_id,
};

/// A live MQTT microservice client.
pub struct MqttClient {
    client: Arc<OnceCell<AsyncClient>>,
    config: MqttClientConfig,
    response_handlers:
        Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<Result<Vec<u8>, TransportError>>>>>,
}

/// Configuration for an MQTT microservice client.
#[derive(Clone, Debug)]
pub struct MqttClientConfig {
    /// MQTT broker URL (e.g., `mqtt://localhost:1883`).
    pub url: String,
    /// Client ID for the MQTT connection.
    pub client_id: String,
    /// Topic prefix.
    pub topic_prefix: String,
}

impl Default for MqttClientConfig {
    fn default() -> Self {
        Self {
            url: "mqtt://localhost:1883".into(),
            client_id: "ironic-client".into(),
            topic_prefix: "ironic".into(),
        }
    }
}

impl MqttClient {
    /// Creates a new MQTT client.
    #[must_use]
    pub fn new(config: MqttClientConfig) -> Self {
        Self {
            client: Arc::new(OnceCell::new()),
            config,
            response_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceClient for MqttClient {
    fn connect(&self) -> ClientFuture<()> {
        let config = self.config.clone();
        let client_cell = Arc::clone(&self.client);
        let handlers = Arc::clone(&self.response_handlers);

        Box::pin(async move {
            let mut mqtt_options = MqttOptions::new(&config.client_id, &config.url, 1883);
            mqtt_options.set_keep_alive(std::time::Duration::from_secs(5));
            let (client, mut eventloop) = AsyncClient::new(mqtt_options, 100);

            client
                .subscribe(
                    &format!("{}/reply/#", config.topic_prefix),
                    QoS::AtMostOnce,
                )
                .await
                .map_err(|e| TransportError(e.to_string()))?;

            let handlers_clone = Arc::clone(&handlers);
            let prefix = config.topic_prefix.clone();
            tokio::spawn(async move {
                while let Ok(notification) = eventloop.poll().await {
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        let topic = publish.topic;
                        if let Some(cid) = topic.strip_prefix(&format!("{prefix}/reply/")) {
                            let mut map = handlers_clone.lock().await;
                            if let Some(tx) = map.remove(cid) {
                                let _ = tx.send(Ok(publish.payload.to_vec()));
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
        let prefix = self.config.topic_prefix.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let client = client_cell
                .get()
                .ok_or_else(|| TransportError("MQTT client not connected".into()))?;
            let correlation_id = generate_correlation_id();
            let (tx, rx) = tokio::sync::oneshot::channel();
            handlers.lock().await.insert(correlation_id.clone(), tx);

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let envelope = serde_json::json!({
                "correlation_id": correlation_id,
                "data": data_str,
            });
            let body = serde_json::to_vec(&envelope)
                .map_err(|e| TransportError(e.to_string()))?;

            client
                .publish(
                    &format!("{prefix}/{pattern}"),
                    QoS::AtMostOnce,
                    false,
                    body,
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
        let client_cell = Arc::clone(&self.client);
        let prefix = self.config.topic_prefix.clone();
        let pattern = pattern.to_string();
        let payload = serde_json::to_vec(data).map_err(|e| TransportError(e.to_string()));

        Box::pin(async move {
            let payload = payload?;
            let client = client_cell
                .get()
                .ok_or_else(|| TransportError("MQTT client not connected".into()))?;

            let data_str = String::from_utf8(payload)
                .map_err(|e| TransportError(format!("payload not utf8: {e}")))?;
            let envelope = serde_json::json!({
                "correlation_id": generate_correlation_id(),
                "data": data_str,
            });
            let body = serde_json::to_vec(&envelope)
                .map_err(|e| TransportError(e.to_string()))?;

            client
                .publish(
                    &format!("{prefix}/{pattern}"),
                    QoS::AtMostOnce,
                    false,
                    body,
                )
                .await
                .map_err(|e| TransportError(e.to_string()))
        })
    }

    fn close(&self) -> ClientFuture<()> {
        Box::pin(async move { Ok(()) })
    }
}

/// A live MQTT microservice server.
pub struct MqttServer {
    config: MqttServerConfig,
    handlers: Arc<std::sync::Mutex<HashMap<String, MessageHandler>>>,
    event_handlers: Arc<std::sync::Mutex<HashMap<String, EventHandler>>>,
}

/// Configuration for an MQTT microservice server.
#[derive(Clone, Debug)]
pub struct MqttServerConfig {
    /// MQTT broker URL.
    pub url: String,
    /// Client ID.
    pub client_id: String,
    /// Topic prefix.
    pub topic_prefix: String,
}

impl Default for MqttServerConfig {
    fn default() -> Self {
        Self {
            url: "mqtt://localhost:1883".into(),
            client_id: "ironic-server".into(),
            topic_prefix: "ironic".into(),
        }
    }
}

impl MqttServer {
    /// Creates a new MQTT server.
    #[must_use]
    pub fn new(config: MqttServerConfig) -> Self {
        Self {
            config,
            handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
            event_handlers: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

impl MicroserviceServer for MqttServer {
    fn listen(&self) -> ServerFuture<()> {
        let config = self.config.clone();
        let handlers = Arc::clone(&self.handlers);
        let event_handlers = Arc::clone(&self.event_handlers);

        Box::pin(async move {
            let mut mqtt_options = MqttOptions::new(&config.client_id, &config.url, 1883);
            mqtt_options.set_keep_alive(std::time::Duration::from_secs(5));
            let (client, mut eventloop) = AsyncClient::new(mqtt_options, 100);

            let registered_patterns: Vec<String> = {
                let h = handlers.lock().unwrap();
                h.keys().cloned().collect()
            };
            let event_patterns: Vec<String> = {
                let h = event_handlers.lock().unwrap();
                h.keys().cloned().collect()
            };

            for pattern in &registered_patterns {
                client
                    .subscribe(
                        &format!("{}/{}", config.topic_prefix, pattern),
                        QoS::AtMostOnce,
                    )
                    .await
                    .map_err(|e| TransportError(e.to_string()))?;
            }
            for pattern in &event_patterns {
                client
                    .subscribe(
                        &format!("{}/{}", config.topic_prefix, pattern),
                        QoS::AtMostOnce,
                    )
                    .await
                    .map_err(|e| TransportError(e.to_string()))?;
            }

            let prefix = config.topic_prefix.clone();
            let handlers_c = Arc::clone(&handlers);
            let event_handlers_c = Arc::clone(&event_handlers);

            tokio::spawn(async move {
                while let Ok(notification) = eventloop.poll().await {
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        let topic = publish.topic;
                        let pattern = topic.strip_prefix(&format!("{prefix}/")).unwrap_or(&topic).to_string();
                        let payload = publish.payload.to_vec();

                        if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&payload) {
                            let correlation_id = parsed["correlation_id"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
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
                                let h = handlers_c.lock().unwrap();
                                h.get(&pattern).cloned()
                            };
                            if let Some(handler) = handler_opt {
                                let result = handler(data, context).await;
                                if let Ok(response) = result {
                                    let _ = client
                                        .publish(
                                            &format!("{prefix}/reply/{cid}"),
                                            QoS::AtMostOnce,
                                            false,
                                            response,
                                        )
                                        .await;
                                }
                            } else {
                                let handler_opt = {
                                    let h = event_handlers_c.lock().unwrap();
                                    h.get(&pattern).cloned()
                                };
                                if let Some(handler) = handler_opt {
                                    let _ = handler(data, context).await;
                                }
                            }
                        }
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
