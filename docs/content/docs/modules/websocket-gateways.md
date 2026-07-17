---
title: WebSocket Gateways
description: Declare WebSocket endpoints with `#[web_socket_gateway]` and broadcast messages with `WsConnections`.
---

# WebSocket Gateways

## What you'll learn

- Declare WebSocket endpoints with `#[web_socket_gateway]`
- Handle incoming messages with `#[subscribe_message]`
- Broadcast messages to clients and rooms with `WsConnections`

---

## Enabling

```toml
ironic = { features = ["realtime"] }
```

Or via the collective feature:

```toml
ironic = { features = ["application-services"] }
```

---

## Declaring a gateway

Use `#[web_socket_gateway("/path")]` on a unit struct. The macro generates `provider_definition()` and `gateway_definition()` methods:

```rust
use ironic::prelude::*;

#[web_socket_gateway("/chat")]
struct ChatGateway;
```

## Message handlers

Define handlers inside a `#[routes]` impl block with `#[subscribe_message("event")]`:

```rust
#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("Echo: {payload}"))
    }
}
```

## Registering the gateway

Add the gateway to your compiled application:

```rust
let app = CompiledHttpApplication::new(container, routes)
    .ws_gateway(ChatGateway::gateway_definition());
```

---

## Broadcasting with WsConnections

`WsConnections` manages connected clients, rooms, and message delivery:

```rust
use ironic::services::ws::{WsConnections, WsMessage};

let connections = WsConnections::new();

// Client connects
let (client_id, mut rx) = connections.connect().await;

// Join a room
connections.join_room("general", client_id).await;

// Broadcast to everyone in a room
connections.broadcast_room("general", WsMessage {
    event: "chat.message".into(),
    data: r#"{"from":"system","text":"Welcome!"}"#.into(),
}).await;

// Send to a specific client
connections.send_to(client_id, WsMessage {
    event: "private".into(),
    data: r#"{"message":"you have a secret"}"#.into(),
}).await;

// Receive messages
while let Some(msg) = rx.recv().await {
    println!("received: {} — {}", msg.event, msg.data);
}
```

### API reference

| Method | Description |
|--------|-------------|
| `connect()` | Register a new client, return `(ClientId, receiver)` |
| `disconnect(id)` | Remove a client |
| `join_room(room, id)` | Subscribe client to a room |
| `leave_room(room, id)` | Unsubscribe client from a room |
| `broadcast_all(msg)` | Send to every connected client |
| `broadcast_room(room, msg)` | Send to all clients in a room |
| `send_to(id, msg)` | Send to a specific client |
| `connected_count()` | Number of connected clients |

---

## Message format

Incoming text frames are parsed as JSON:

```json
{"event": "message", "data": {"text": "hello"}}
```

Use `parse_incoming()` to parse a raw text frame:

```rust
use ironic::services::ws::parse_incoming;

let msg = parse_incoming(r#"{"event":"message","data":{"text":"hello"}}"#)?;
assert_eq!(msg.event, "message");
```

---

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Gateway not receiving connections | Verify `realtime` feature is enabled and the gateway is registered with `.ws_gateway(...)` |
| `#[subscribe_message]` handler not called | The marker attribute is consumed by `#[routes]` but runtime dispatch is not yet implemented — the platform echoes text frames |
| Forgetting to hold `WsConnections` | `WsConnections` uses `Arc` internally and is cheaply cloneable |
| Broadcasting after client disconnects | `send_to()` silently fails — check `connected_count()` or handle errors on the receiver side |

## What you learned

- [x] `#[web_socket_gateway("/path")]` declares a WebSocket endpoint
- [x] `#[subscribe_message("event")]` annotates handler methods
- [x] `WsConnections` manages clients, rooms, and broadcasting
- [x] Messages are JSON `{"event": "...", "data": ...}` frames
- [x] Register with `.ws_gateway(...)` on `CompiledHttpApplication`
