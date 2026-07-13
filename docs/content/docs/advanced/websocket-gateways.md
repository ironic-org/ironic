---
title: WebSocket gateways
description: Real-time bidirectional communication with gateway classes, rooms, and event-driven message routing.
---

# WebSocket gateways

Enable `realtime` to define server-side WebSocket gateways that handle connection lifecycle
events and route incoming messages to handler methods.

```toml
ironic = { features = ["realtime"] }
```

## Defining a gateway

```rust
use ironic::{web_socket_gateway, subscribe_message, WsGatewayDefinition};

#[web_socket_gateway("/ws")]
struct ChatGateway;
```

## Message routing

Use `#[subscribe_message("event")]` on gateway methods to route incoming messages by event name:

```rust
#[web_socket_gateway("/ws")]
struct ChatGateway;

#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("Echo: {}", payload))
    }

    #[subscribe_message("join-room")]
    async fn on_join(&self, room: String) -> Result<(), HttpError> {
        // Join the client to a named room
        Ok(())
    }
}
```

The event name in `#[subscribe_message("event")]` maps to the `event` field in the incoming
JSON payload. The method receives the deserialized payload and returns a response that is sent
back to the calling client only.

## Connection lifecycle

Gateways track connected clients automatically and support room management:

```rust
use ironic::services::realtime::{WsServer, ConnectionId, RoomId};

// Broadcasting to all connected clients
server.broadcast_all("user-joined".to_string(), &payload).await;

// Broadcasting to members of a room
server.broadcast_room(RoomId::new("lobby"), "room-message".to_string(), &payload).await;

// Sending to a specific client
server.send_to(connection_id, "private".to_string(), &payload).await;
```

## Room management

```rust
// Join a room
server.join(connection_id, RoomId::new("lobby")).await;

// Leave a room
server.leave(connection_id, RoomId::new("lobby")).await;
```

Disconnected clients are removed from all rooms automatically.

## Incoming message format

Clients send JSON messages with an `event` field that matches the `#[subscribe_message]`
attribute and a `data` field that is deserialized into the method parameter:

```json
{ "event": "message", "data": "Hello, world!" }
```

Messages with unknown events or malformed JSON are silently discarded.
