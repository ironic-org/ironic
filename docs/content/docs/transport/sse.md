---
title: Server-Sent Events (SSE)
description: One-way real-time event streaming from server to client.
---

# Server-Sent Events (SSE)

SSE (Server-Sent Events) provides a one-way, text-based protocol for streaming events from server to client over a single HTTP connection. Unlike WebSocket, SSE uses standard HTTP and supports automatic reconnection via `Last-Event-ID`.

## Enabling SSE

Enable the `sse` feature:

```toml
[dependencies]
ironic = { version = "1.0", features = ["sse"] }
```

## Programmatic SSE

Create an SSE endpoint by pairing a sender with a response stream:

```rust
use ironic::services::sse::{SseRoute, SseConfig, sse_endpoint};

let config = SseConfig::default();
let (tx, stream) = sse_endpoint(config);

// tx can be cloned and sent to other parts of the application
let route = SseRoute::new(tx, Arc::new(SseConfig::default()), Arc::default());

// In a route handler:
route.send(axum::response::sse::Event::default().data("hello")).await.unwrap();
```

The returned `Sse<SseStream>` can be used directly as an Axum response.

## Configuration

`SseConfig` supports the following options:

| Field | Type | Default | Description |
|---|---|---|---|
| `reconnect_buffer_size` | `usize` | `1024` | Max events retained for reconnection |
| `keep_alive_interval` | `Duration` | `15s` | Interval for keep-alive comments |
| `event_id_prefix` | `String` | `"ev-"` | Prefix for auto-generated event IDs |

```rust
let config = SseConfig {
    reconnect_buffer_size: 512,
    keep_alive_interval: Duration::from_secs(30),
    event_id_prefix: "msg-".into(),
};
```

## Reconnection

SSE clients automatically reconnect when the connection drops. The server assigns each event a sequential ID. When a client reconnects with a `Last-Event-ID` header, events after that ID are replayed from the buffer.

## SseRoute

`SseRoute` is the server-side handle for sending events:

```rust
pub struct SseRoute {
    sender: mpsc::Sender<Result<Event, Infallible>>,
}

impl SseRoute {
    pub async fn send(&self, event: Event) -> Result<(), SseError>;
}
```

Cloning `SseRoute` creates a handle to the same event stream, allowing multiple parts of the application to push events to the same client.

## SSE vs WebSocket

| Feature | SSE | WebSocket |
|---|---|---|
| Direction | Server → Client | Bidirectional |
| Protocol | HTTP | WS |
| Reconnection | Built-in | Manual |
| Binary data | Text only | Text + Binary |
| Browser support | Native | Native |
| Use case | Notifications, feeds | Chat, gaming, collaboration |

## Error Handling

`SseError` provides:

```rust
pub enum SseError {
    ClientDisconnected,
}
```

Handle errors when sending:

```rust
match route.send(event).await {
    Ok(()) => tracing::debug!("event sent"),
    Err(SseError::ClientDisconnected) => tracing::warn!("client disconnected"),
}
```
