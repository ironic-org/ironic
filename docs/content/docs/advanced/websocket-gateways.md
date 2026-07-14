---
title: WebSocket Gateways
description: Add real-time communication to your API — WebSocket connections, rooms, and broadcasting.
---

# WebSocket Gateways

## What you'll learn

- Create WebSocket endpoints
- Handle incoming messages
- Broadcast messages to rooms
- Manage client connections

Enable in `Cargo.toml`:

```toml
ironic = { features = ["realtime"] }
```

---

## Quick start

```rust
use ironic::{HttpError, subscribe_message, web_socket_gateway};

#[web_socket_gateway("/chat")]
pub struct ChatGateway;

#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("Echo: {payload}"))
    }
}
```

Now clients can connect:

```javascript
const ws = new WebSocket("ws://localhost:3000/chat");
ws.onopen = () => ws.send(JSON.stringify({ event: "message", data: "Hello!" }));
ws.onmessage = (e) => console.log("Server says:", e.data);
// → Server says: Echo: Hello!
```

## Rooms and broadcasting

Group clients into rooms:

```rust
use ironic::services::ws::WebSocketServer;

impl ChatGateway {
    #[subscribe_message("join")]
    async fn join_room(&self, room: String, client_id: String) {
        WebSocketServer::join_room(&room, &client_id);
    }

    #[subscribe_message("broadcast")]
    async fn broadcast(&self, room: String, message: String) {
        WebSocketServer::to_room(&room).send(message);
    }
}
```

### Client flow

```javascript
// Client A joins the "lobby" room
ws.send(JSON.stringify({ event: "join", data: "lobby" }));

// Client B broadcasts to the lobby
ws.send(JSON.stringify({ event: "broadcast", data: { room: "lobby", message: "Hello everyone!" }}));

// Client A receives: "Hello everyone!"
```

## Try it yourself

1. Create a `ChatGateway` at `/ws/chat`
2. Add a "message" event handler that echoes back
3. Connect with a browser WebSocket client
4. Send a message and verify the echo

## What you learned

- [x] `#[web_socket_gateway]` creates WebSocket endpoints
- [x] `#[subscribe_message]` handles incoming messages
- [x] Rooms group clients for broadcasting
- [x] WebSocket connections work alongside regular HTTP routes
