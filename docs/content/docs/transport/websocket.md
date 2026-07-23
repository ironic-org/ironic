---
title: WebSocket Transport
description: Real-time bidirectional communication via WebSocket gateways.
---

# WebSocket Transport

WebSocket support provides real-time, bidirectional communication between clients and your Ironic application. Built on the WebSocket gateway pattern, it supports rooms, broadcasting, authentication, and connection lifecycle management.

## Enabling WebSocket

Enable the `realtime` feature:

```toml
[dependencies]
ironic = { version = "1.0", features = ["realtime"] }
```

## WebSocket Gateways

A gateway handles WebSocket connections for a specific path:

```rust
use ironic::*;

#[web_socket_gateway("/ws/chat")]
struct ChatGateway;

#[subscribe_message("chat.message")]
async fn handle_message(
    message: ChatMessage,
    context: WebSocketContext,
) -> Result<(), WebSocketError> {
    context.broadcast_to_room("general", &message).await?;
    Ok(())
}
```

## Connection Lifecycle

| Phase | Description |
|-------|-------------|
| `on_connect` | Authenticate and validate the connection |
| `on_message` | Handle incoming messages |
| `on_close` | Clean up resources on disconnect |

## Rooms & Broadcasting

Messages can be scoped to rooms for efficient fan-out:

```rust
// Join a room
context.join("room:123").await?;

// Broadcast to room
context.broadcast_to_room("room:123", &event).await?;

// Send to specific client
context.send(client_id, &private_message).await?;
```

## Authentication

WebSocket connections can be authenticated during the handshake:

```rust
#[on_connect]
async fn authenticate(token: String) -> Result<Principal, WebSocketError> {
    let principal = auth::verify_token(&token)?;
    Ok(principal)
}
```

## SSE (Server-Sent Events)

For unidirectional server-to-client streaming, SSE channels provide a simpler alternative:

```rust
use ironic::services::SseChannel;

let channel = SseChannel::new(100);
channel.send(&event).await?;
// Client connects via GET /events
```

## When to Use What

| Protocol | Direction | Best For |
|----------|-----------|----------|
| WebSocket | Bidirectional | Chat, live collaboration, gaming |
| SSE | Server → Client | Notifications, live feeds, status updates |
| HTTP Polling | Client → Server | Simple request-response, CRUD APIs |

## Roadmap

- **WebSocket compression** (per-message deflate)
- **Automatic reconnection helpers** for client SDKs
- **WebSocket health checks** and connection metrics
