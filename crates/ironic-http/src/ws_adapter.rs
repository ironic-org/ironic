#![allow(clippy::type_complexity)]
//! WebSocket adapter abstraction for platform-independent WebSocket support.

use std::{future::Future, pin::Pin};

/// A platform-neutral WebSocket connection.
pub trait WsConnection: Send + Sync + 'static {
    /// Sends a message to the connected client.
    fn send(&self, message: WsMessage) -> Pin<Box<dyn Future<Output = Result<(), WsError>> + Send>>;
    /// Receives the next message from the client.
    fn recv(&self) -> Pin<Box<dyn Future<Output = Option<Result<WsMessage, WsError>>> + Send>>;
    /// Closes the connection.
    fn close(&self) -> Pin<Box<dyn Future<Output = Result<(), WsError>> + Send>>;
}

/// A WebSocket message.
#[derive(Clone, Debug)]
pub enum WsMessage {
    /// Text message.
    Text(String),
    /// Binary message.
    Binary(Vec<u8>),
    /// Ping message.
    Ping(Vec<u8>),
    /// Pong message.
    Pong(Vec<u8>),
}

/// A WebSocket error.
#[derive(Clone, Debug, thiserror::Error)]
pub enum WsError {
    /// Connection closed.
    #[error("WebSocket connection closed")]
    ConnectionClosed,
    /// Protocol error.
    #[error("WebSocket protocol error: {0}")]
    Protocol(String),
    /// Internal error.
    #[error("WebSocket internal error: {0}")]
    Internal(String),
}

/// A platform-neutral WebSocket adapter.
///
/// Implement this trait to support WebSocket on different platforms.
pub trait WsAdapter: Send + Sync + 'static {
    /// The connection type for this adapter.
    type Connection: WsConnection;
    /// Upgrades an HTTP request to a WebSocket connection.
    fn upgrade(
        &self,
        request: crate::Request,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Connection, WsError>> + Send>>;
}
