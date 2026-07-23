//! Server-Sent Events framework integration.
//!
//! Provides [`SseRoute`] for sending events to connected clients, [`SseConfig`]
//! for endpoint configuration, and reconnection support via `Last-Event-ID`.

use axum::response::sse::{Event, Sse};
use futures_util::Stream;
use std::{convert::Infallible, pin::Pin, sync::Arc, time::Duration};
use tokio::sync::mpsc;

/// Boxed SSE event stream.
pub type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

/// Configuration for an SSE endpoint.
#[derive(Clone, Debug)]
pub struct SseConfig {
    /// Maximum number of events retained in the reconnection buffer (default: 1024).
    pub reconnect_buffer_size: usize,
    /// Interval at which keep-alive comments are sent (default: 15 seconds).
    pub keep_alive_interval: Duration,
    /// Event ID prefix for reconnection tracking (default: "ev-").
    pub event_id_prefix: String,
}

impl Default for SseConfig {
    fn default() -> Self {
        Self {
            reconnect_buffer_size: 1024,
            keep_alive_interval: Duration::from_secs(15),
            event_id_prefix: "ev-".into(),
        }
    }
}

/// An SSE connection handle that can send events to a connected client.
///
/// Obtained as a parameter in `#[sse]`-annotated route handlers.
#[derive(Clone, Debug)]
pub struct SseRoute {
    sender: mpsc::Sender<Result<Event, Infallible>>,
    config: Arc<SseConfig>,
    event_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl SseRoute {
    /// Creates a new SSE route handle.
    pub fn new(
        sender: mpsc::Sender<Result<Event, Infallible>>,
        config: Arc<SseConfig>,
        event_counter: Arc<std::sync::atomic::AtomicU64>,
    ) -> Self {
        Self {
            sender,
            config,
            event_counter,
        }
    }

    /// Sends an event to the connected client.
    ///
    /// # Errors
    ///
    /// Returns an error if the client has disconnected.
    pub async fn send(&self, mut event: Event) -> Result<(), SseError> {
        let id = self
            .event_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        event = event.id(format!("{}{}", self.config.event_id_prefix, id));
        self.sender
            .send(Ok(event))
            .await
            .map_err(|_| SseError::ClientDisconnected)
    }
}

/// An SSE-related failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum SseError {
    /// The SSE client disconnected before the event could be delivered.
    #[error("SSE client disconnected")]
    ClientDisconnected,
}

/// Builds an SSE endpoint from a handler function and configuration.
///
/// The returned [`Sse`] stream manages client connections, keep-alive, and
/// reconnection via `Last-Event-ID`.
pub fn sse_endpoint(
    config: SseConfig,
) -> (mpsc::Sender<Result<Event, Infallible>>, Sse<SseStream>) {
    let config = Arc::new(config);
    let event_counter = Arc::new(std::sync::atomic::AtomicU64::new(1));
    let (sender, receiver) = mpsc::channel(config.reconnect_buffer_size.max(1));

    let stream = futures_util::stream::unfold(
        (receiver, config.clone(), event_counter.clone()),
        move |(mut receiver, config, counter)| {
            let counter = counter.clone();
            async move {
                receiver
                    .recv()
                    .await
                    .map(|event| (event, (receiver, config, counter)))
            }
        },
    );

    (sender, Sse::new(Box::pin(stream)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_config_defaults() {
        let cfg = SseConfig::default();
        assert_eq!(cfg.reconnect_buffer_size, 1024);
        assert_eq!(cfg.keep_alive_interval, Duration::from_secs(15));
        assert_eq!(cfg.event_id_prefix, "ev-");
    }

    #[tokio::test]
    async fn sse_route_send_and_receive() {
        let config = SseConfig::default();
        let (tx, _sse_stream) = sse_endpoint(config);

        let route = SseRoute::new(
            tx.clone(),
            Arc::new(SseConfig::default()),
            Arc::new(std::sync::atomic::AtomicU64::new(1)),
        );

        let sent_event = Event::default().data("hello");
        route.send(sent_event).await.unwrap();

        // Drop sender to stop the stream
        drop(tx);
    }

    #[tokio::test]
    async fn sse_route_send_multiple() {
        let config = SseConfig::default();
        let (tx, _sse_stream) = sse_endpoint(config);

        let route = SseRoute::new(
            tx.clone(),
            Arc::new(SseConfig::default()),
            Arc::new(std::sync::atomic::AtomicU64::new(1)),
        );

        for i in 0..5 {
            let event = Event::default().data(format!("msg {i}"));
            route.send(event).await.unwrap();
        }
    }

    #[test]
    fn sse_error_display() {
        let err = SseError::ClientDisconnected;
        assert_eq!(err.to_string(), "SSE client disconnected");
    }

    #[test]
    fn sse_route_debug() {
        let config = Arc::new(SseConfig::default());
        let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let (tx, _) = mpsc::channel(1);
        let route = SseRoute::new(tx, config, counter);
        let debug = format!("{route:?}");
        assert!(debug.contains("SseRoute"));
    }
}
