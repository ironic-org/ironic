#![allow(deprecated)]

//! MQTT transport backend (config stub).
//! Requires the `transport-mqtt` feature and a running MQTT broker.

use crate::distributed::microservices::{Transport, TransportError, TransportFuture, Envelope};

/// Configuration for an MQTT transport.
#[cfg(feature = "transport-mqtt")]
#[derive(Clone, Debug)]
pub struct MqttTransportConfig {
    /// MQTT broker URL.
    pub url: String,
    /// Topic prefix.
    pub topic_prefix: String,
}

#[cfg(feature = "transport-mqtt")]
impl MqttTransportConfig {
    /// Creates a new MQTT transport configuration.
    #[must_use]
    pub fn new(url: impl Into<String>, topic_prefix: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            topic_prefix: topic_prefix.into(),
        }
    }
}

/// An MQTT transport (requires live connection).
#[cfg(feature = "transport-mqtt")]
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MqttTransport {
    config: MqttTransportConfig,
}

#[cfg(feature = "transport-mqtt")]
impl Transport for MqttTransport {
    fn send(&self, _envelope: Envelope) -> TransportFuture<'_, ()> {
        Box::pin(async move {
            Err(TransportError(
                "MQTT transport requires a live connection".into(),
            ))
        })
    }

    fn receive(&self) -> TransportFuture<'_, Option<Envelope>> {
        Box::pin(async move {
            Err(TransportError(
                "MQTT transport requires a live connection".into(),
            ))
        })
    }
}
