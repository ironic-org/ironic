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
pub trait WebSocketHandler: Clone + Send + Sync + 'static {
    /// Runs one connection until it closes.
    fn handle(&self, socket: WebSocket) -> WebSocketFuture;
}

/// Completes a native Axum WebSocket upgrade with an application handler.
pub fn websocket<H: WebSocketHandler>(
    upgrade: WebSocketUpgrade,
    handler: H,
) -> axum::response::Response {
    upgrade.on_upgrade(move |socket| handler.handle(socket))
}

/// Boxed stream used by [`sse_channel`].
pub type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;

/// Creates a bounded SSE sender and response stream.
pub fn sse_channel(capacity: usize) -> (mpsc::Sender<Event>, Sse<SseStream>) {
    let (sender, receiver) = mpsc::channel(capacity.max(1));
    let stream = futures_util::stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (Ok(event), receiver))
    });
    (sender, Sse::new(Box::pin(stream)))
}
