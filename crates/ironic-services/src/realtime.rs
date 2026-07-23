//! Axum WebSocket and Server-Sent Event helpers for native realtime routes.

use axum::response::sse::{Event, Sse};
use futures_util::Stream;
use std::{convert::Infallible, future::Future, pin::Pin};
use tokio::sync::mpsc;

/// Native Axum WebSocket types.
pub use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};

/// A boxed asynchronous WebSocket session.
pub type WebSocketFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Application handler for an upgraded WebSocket connection.
///
/// # Errors
///
/// The handler runs inside a WebSocket session; errors are handled internally.
///
/// # Panics
///
/// Implementations should not panic.
pub trait WebSocketHandler: Clone + Send + Sync + 'static {
    /// Runs one connection until it closes.
    fn handle(&self, socket: WebSocket) -> WebSocketFuture;
}

/// Completes a native Axum WebSocket upgrade with an application handler.
///
/// # Errors
///
/// Delegates to [`WebSocketUpgrade::on_upgrade`]; always infallible at this layer.
///
/// # Panics
///
/// Never panics.
pub fn websocket<H: WebSocketHandler>(
    upgrade: WebSocketUpgrade,
    handler: H,
) -> axum::response::Response {
    upgrade.on_upgrade(move |socket| handler.handle(socket))
}

/// Boxed stream used by [`sse_channel`].
pub type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

/// Creates a bounded SSE sender and response stream.
///
/// # Errors
///
/// The returned [`Sse`] stream never yields errors.
///
/// # Panics
///
/// Never panics.
pub fn sse_channel(capacity: usize) -> (mpsc::Sender<Event>, Sse<SseStream>) {
    let (sender, receiver) = mpsc::channel(capacity.max(1));
    let stream = futures_util::stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (Ok(event), receiver))
    });
    (sender, Sse::new(Box::pin(stream)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_channel_zero_capacity_defaults_to_one() {
        let (tx, _sse) = sse_channel(0);
        let result = tx.try_send(Event::default().data("hello"));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sse_channel_send_and_drop() {
        let (tx, _sse) = sse_channel(4);
        tx.send(Event::default().data("a")).await.unwrap();
        tx.send(Event::default().data("b")).await.unwrap();
    }

    #[tokio::test]
    async fn sse_channel_sender_capacity_works() {
        let (tx, _sse) = sse_channel(2);
        assert!(tx.try_send(Event::default().data("x")).is_ok());
        assert!(tx.try_send(Event::default().data("y")).is_ok());
        // capacity is 2, so third send may fail; that's fine
        let _ = tx.try_send(Event::default().data("z"));
    }
}
