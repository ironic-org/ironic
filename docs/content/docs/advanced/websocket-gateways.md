---
title: WebSocket Gateways
description: Add real-time communication to your API — WebSocket connections, rooms, and broadcasting.
---

# WebSocket Gateways

## What you'll learn

- Create WebSocket endpoints with `#[web_socket_gateway]`
- Handle incoming messages with `#[subscribe_message]`
- Manage connection lifecycle (open, message, close)
- Authenticate WebSocket connections
- Broadcast messages to rooms
- Handle errors gracefully in WebSocket handlers
- Decide between WebSocket and SSE

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

## Message handlers

Use `#[subscribe_message("event")]` to handle incoming messages:

```rust
#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("Echo: {payload}"))
    }
}
```

Each handler receives the JSON data field as its argument. The event name in the attribute must match the `"event"` field in the incoming JSON: `{"event": "message", "data": "Hello!"}`.

## Authentication

Authenticate clients during the WebSocket upgrade by validating a token in the handshake request. Use a `Guard` on the gateway struct:

```rust
#[web_socket_gateway("/chat")]
#[guard(JwtGuard)]
struct ChatGateway;
```

The guard runs during the HTTP upgrade handshake. If the guard returns `Deny`, the connection is rejected with 403 before the WebSocket is established.

## Rooms and broadcasting

Group clients into rooms to send targeted messages:

```rust
use ironic::services::ws::WebSocketServer;

impl ChatGateway {
    #[subscribe_message("join")]
    async fn join_room(&self, room: String, client_id: String) {
        WebSocketServer::join_room(&room, &client_id);
    }

    #[subscribe_message("leave")]
    async fn leave_room(&self, room: String, client_id: String) {
        WebSocketServer::leave_room(&room, &client_id);
    }

    #[subscribe_message("broadcast")]
    async fn broadcast(&self, room: String, message: String, client_id: String) {
        WebSocketServer::to_room(&room)
            .except(&client_id)        // Don't send back to sender
            .send(message);
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

Rooms are dynamic — created when the first client joins, destroyed when the last leaves.

## Error handling

When a handler panics or returns an `Err`, the connection is closed with an error frame:

```rust
#[subscribe_message("unsafe_op")]
async fn risky_handler(&self, data: String) -> Result<String, HttpError> {
    // Returning Err closes the connection cleanly with a close frame
    if data.is_empty() {
        return Err(HttpError::bad_request("EMPTY_DATA", "Data must not be empty"));
    }
    Ok(data.to_uppercase())
}
```

The framework catches panics in handlers and closes the affected connection. Other clients on the gateway continue unaffected — one bad handler does not bring down the whole gateway.

## Client-side reconnection

WebSocket connections can drop. Implement exponential backoff on the client:

```javascript
function connect(path, maxRetries = 5) {
    let retries = 0;
    let delay = 1000;

    function attempt() {
        const ws = new WebSocket(path);

        ws.onclose = () => {
            if (retries < maxRetries) {
                retries++;
                setTimeout(attempt, delay);
                delay *= 2;  // 1s → 2s → 4s → 8s → 16s
            }
        };

        ws.onopen = () => { retries = 0; delay = 1000; };
        ws.onmessage = (e) => console.log(e.data);
    }

    attempt();
}
```

## WebSocket vs SSE

| Criteria | WebSocket | Server-Sent Events |
|----------|-----------|---------------------|
| Direction | Bidirectional | Server → client only |
| Protocol | Custom (ws://, wss://) | Standard HTTP |
| Auto-reconnect | Manual | Built-in (`EventSource`) |
| Browser support | All modern browsers | All modern browsers |
| Use case | Chat, gaming, collaborative editing | Live feeds, notifications, status updates |
| Overhead | Lower per-message overhead | Slightly higher (HTTP framing) |

Choose **WebSocket** when the client needs to send data to the server in real time. Choose **SSE** when you only push updates from server to client and want simpler infrastructure.

## Common mistakes

| Mistake | Why it hurts | Fix |
|---------|-------------|-----|
| Not checking auth in `on_open` | Unauthenticated clients can join rooms | Validate tokens before accepting the connection |
| Forgetting `leave_room` on close | Rooms accumulate stale client references | Call `leave_room` in `#[on_close]` |
| Blocking in a handler | Blocks the event loop for all connections | Use `async` handlers; offload CPU work to a task queue |
| No reconnection logic | Clients silently disconnect on network blips | Implement exponential backoff on the client side |
| Broadcasting without `.except()` | Sender receives their own message back | Use `.except(&sender_id)` when broadcasting |

## Try it yourself

1. Create a `ChatGateway` at `/ws/chat`
2. Add `on_open`, `on_message`, and `on_close` handlers
3. Add a "join" event that puts clients into rooms
4. Connect with a browser WebSocket client
5. Send a message and verify the echo

## What you learned

- [x] `#[web_socket_gateway]` creates WebSocket endpoints
- [x] `#[subscribe_message]` handles incoming messages
- [x] `#[on_open]` and `#[on_close]` manage the connection lifecycle
- [x] `HttpRequest` in `on_open` enables token-based authentication
- [x] Rooms group clients for broadcasting with `.to_room()`
- [x] Errors in handlers close the connection cleanly
- [x] Choose WebSocket for bidirectional, SSE for server-to-client only
