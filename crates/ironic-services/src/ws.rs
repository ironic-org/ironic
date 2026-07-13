use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, mpsc};

/// Unique identifier for a connected WebSocket client.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ClientId(u64);

static NEXT_CLIENT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl ClientId {
    fn new() -> Self {
        Self(NEXT_CLIENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// A message sent from a gateway to a connected client.
#[derive(Clone, Debug)]
pub struct WsMessage {
    /// The event name.
    pub event: String,
    /// JSON-encoded payload.
    pub data: String,
}

/// Manages all connected WebSocket clients and rooms.
#[derive(Clone)]
pub struct WsConnections {
    clients: Arc<RwLock<HashMap<ClientId, mpsc::UnboundedSender<WsMessage>>>>,
    rooms: Arc<RwLock<HashMap<String, Vec<ClientId>>>>,
}

impl WsConnections {
    /// Creates an empty connection manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a new client and returns its ID and a receiver for outgoing messages.
    pub async fn connect(&self) -> (ClientId, mpsc::UnboundedReceiver<WsMessage>) {
        let id = ClientId::new();
        let (sender, receiver) = mpsc::unbounded_channel();
        self.clients.write().await.insert(id, sender);
        (id, receiver)
    }

    /// Removes a client from all rooms and the connection map.
    pub async fn disconnect(&self, id: ClientId) {
        self.clients.write().await.remove(&id);
        let mut rooms = self.rooms.write().await;
        for members in rooms.values_mut() {
            members.retain(|c| *c != id);
        }
    }

    /// Adds a client to a named room.
    pub async fn join_room(&self, room: &str, client: ClientId) {
        let mut rooms = self.rooms.write().await;
        rooms.entry(room.to_string()).or_default().push(client);
    }

    /// Removes a client from a named room.
    pub async fn leave_room(&self, room: &str, client: ClientId) {
        let mut rooms = self.rooms.write().await;
        if let Some(members) = rooms.get_mut(room) {
            members.retain(|c| *c != client);
        }
    }

    /// Broadcasts a message to every connected client.
    pub async fn broadcast_all(&self, message: WsMessage) {
        let clients = self.clients.read().await;
        for sender in clients.values() {
            let _ = sender.send(message.clone());
        }
    }

    /// Broadcasts a message to all clients in a named room.
    pub async fn broadcast_room(&self, room: &str, message: WsMessage) {
        let rooms = self.rooms.read().await;
        if let Some(members) = rooms.get(room) {
            let clients = self.clients.read().await;
            for client_id in members {
                if let Some(sender) = clients.get(client_id) {
                    let _ = sender.send(message.clone());
                }
            }
        }
    }

    /// Broadcasts a message to a specific client.
    pub async fn send_to(&self, client: ClientId, message: WsMessage) {
        let clients = self.clients.read().await;
        if let Some(sender) = clients.get(&client) {
            let _ = sender.send(message);
        }
    }

    /// Returns the number of connected clients.
    pub async fn connected_count(&self) -> usize {
        self.clients.read().await.len()
    }
}

impl Default for WsConnections {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed WebSocket frame from a client.
#[derive(Clone, Debug)]
pub struct IncomingMessage {
    /// The event name extracted from the JSON payload.
    pub event: String,
    /// The raw JSON data payload.
    pub data: String,
}

/// Attempts to parse an incoming WebSocket text message as `{"event": "...", "data": {...}}`.
pub fn parse_incoming(text: &str) -> Result<IncomingMessage, String> {
    let value: crate::__private::serde_json::Value =
        crate::__private::serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?;
    let event = value
        .get("event")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing `event` field".to_string())?
        .to_string();
    let data = value
        .get("data")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_string());
    Ok(IncomingMessage { event, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_and_disconnect_tracks_clients() {
        let connections = WsConnections::new();
        assert_eq!(connections.connected_count().await, 0);

        let (id, _receiver) = connections.connect().await;
        assert_eq!(connections.connected_count().await, 1);

        connections.disconnect(id).await;
        assert_eq!(connections.connected_count().await, 0);
    }

    #[tokio::test]
    async fn broadcast_all_delivers_to_all_clients() {
        let connections = WsConnections::new();
        let (_id1, mut rx1) = connections.connect().await;
        let (_id2, mut rx2) = connections.connect().await;

        let msg = WsMessage {
            event: "test".to_string(),
            data: "{}".to_string(),
        };
        connections.broadcast_all(msg).await;

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }

    #[tokio::test]
    async fn disconnected_client_no_longer_receives() {
        let connections = WsConnections::new();
        let (id, mut rx) = connections.connect().await;
        connections.disconnect(id).await;

        let msg = WsMessage {
            event: "test".to_string(),
            data: "{}".to_string(),
        };
        connections.broadcast_all(msg).await;

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn room_join_and_broadcast() {
        let connections = WsConnections::new();
        let (id, mut rx) = connections.connect().await;
        connections.join_room("general", id).await;
        connections.join_room("random", id).await;

        let msg = WsMessage {
            event: "room.msg".to_string(),
            data: "{}".to_string(),
        };
        connections.broadcast_room("general", msg).await;

        assert!(rx.try_recv().is_ok());
    }

    #[tokio::test]
    async fn room_leave_stops_receiving() {
        let connections = WsConnections::new();
        let (id, mut rx) = connections.connect().await;
        connections.join_room("general", id).await;
        connections.leave_room("general", id).await;

        let msg = WsMessage {
            event: "room.msg".to_string(),
            data: "{}".to_string(),
        };
        connections.broadcast_room("general", msg).await;

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn send_to_specific_client() {
        let connections = WsConnections::new();
        let (id, mut rx) = connections.connect().await;

        let msg = WsMessage {
            event: "private".to_string(),
            data: "\"ping\"".to_string(),
        };
        connections.send_to(id, msg).await;

        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn parse_incoming_valid_message() {
        let result = parse_incoming(r#"{"event":"chat.message","data":{"text":"hello"}}"#);
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.event, "chat.message");
        assert!(msg.data.contains("hello"));
    }

    #[test]
    fn parse_incoming_missing_event() {
        let result = parse_incoming(r#"{"data":{}}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing"));
    }

    #[test]
    fn parse_incoming_invalid_json() {
        let result = parse_incoming("not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[tokio::test]
    async fn broadcast_room_excludes_non_members() {
        let connections = WsConnections::new();
        let (member_id, mut member_rx) = connections.connect().await;
        let (_non_member_id, mut non_member_rx) = connections.connect().await;
        connections.join_room("general", member_id).await;

        let msg = WsMessage {
            event: "update".to_string(),
            data: "{}".to_string(),
        };
        connections.broadcast_room("general", msg).await;

        assert!(member_rx.try_recv().is_ok());
        assert!(non_member_rx.try_recv().is_err());
    }
}
