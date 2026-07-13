//! Transport-neutral microservice envelopes and duplex in-memory endpoints.

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
