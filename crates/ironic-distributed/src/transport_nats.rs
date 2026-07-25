#![allow(deprecated)]

//! NATS transport backend (config stub).
//! Requires the `transport-nats` feature and a running NATS server.

use crate::distributed::microservices::{Transport, TransportError, TransportFuture, Envelope};

/// Configuration for a NATS transport.
#[cfg(feature = "transport-nats")]
#[derive(Clone, Debug)]
pub struct NatsTransportConfig {
    /// NATS server URL.
    pub url: String,
    /// Subject prefix.
    pub subject_prefix: String,
}

#[cfg(feature = "transport-nats")]
impl NatsTransportConfig {
    /// Creates a new NATS transport configuration.
    #[must_use]
    pub fn new(url: impl Into<String>, subject_prefix: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            subject_prefix: subject_prefix.into(),
        }
    }
}

/// A NATS transport (requires live connection).
#[cfg(feature = "transport-nats")]
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct NatsTransport {
    config: NatsTransportConfig,
}

#[cfg(feature = "transport-nats")]
impl Transport for NatsTransport {
    fn send(&self, _envelope: Envelope) -> TransportFuture<'_, ()> {
        Box::pin(async move {
            Err(TransportError(
                "NATS transport requires a live connection".into(),
            ))
        })
    }

    fn receive(&self) -> TransportFuture<'_, Option<Envelope>> {
        Box::pin(async move {
            Err(TransportError(
                "NATS transport requires a live connection".into(),
            ))
        })
    }
}
