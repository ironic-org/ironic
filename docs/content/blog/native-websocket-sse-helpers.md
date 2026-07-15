---
title: "Native WebSocket and SSE Helpers — adapter-level realtime escapes"
description: "When the #[web_socket_gateway] macro is too much: how Ironic's native Axum WebSocket and SSE helpers give you raw control over realtime connections."
date: "2026-07-15"
author: "Ironic Team"
---

# Native WebSocket and SSE Helpers — adapter-level realtime escapes

The `#[web_socket_gateway]` macro handles the common case: a structured gateway with named events, room membership, and broadcast to connected clients. But sometimes you need to bypass the macro — a raw WebSocket upgrade with custom handshake logic, a one-off SSE stream for a progress bar, or a connection whose lifecycle doesn't fit the gateway model.

Ironic's `realtime` module at `ironic-services/src/realtime.rs` provides low-level Axum helpers for exactly these situations. They're not a replacement for the macro — they're an escape hatch to the native layer when you need it.

---

## `WebSocketHandler`: type-safe, outside the pipeline

The trait at `realtime.rs:15-18` is minimal:

```rust
pub trait WebSocketHandler: Clone + Send + Sync + 'static {
    fn handle(&self, socket: WebSocket) -> WebSocketFuture;
}

pub type WebSocketFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
```

Implement this trait on any struct, and you have a reusable WebSocket session handler. The `socket` parameter is an Axum `WebSocket` — a stream of `Message` variants (Text, Binary, Ping, Pong, Close). You own the read/write loop entirely. No event routing, no room management, no `WsConnections` — just you and the socket.

The `Clone` bound exists because the handler is passed to `axum::response::Response` as a cloneable value. The `WebSocketFuture` is a boxed, pinned future — necessary because the `on_upgrade` callback requires a single concrete future type, and boxing erases the specific async block.

---

## The `websocket()` function: completing the upgrade dance

The Axum WebSocket upgrade flow has a specific pattern. You extract `WebSocketUpgrade` from the request, call `.on_upgrade(|socket| async { ... })`, and return the resulting response. Ironic wraps this in a one-liner at line 21:

```rust
pub fn websocket<H: WebSocketHandler>(
    upgrade: WebSocketUpgrade,
    handler: H,
) -> axum::response::Response {
    upgrade.on_upgrade(move |socket| handler.handle(socket))
}
```

This is a convenience, but an important one: it enforces the `WebSocketHandler` trait bound, which means your handler is type-checked at compile time. You can't accidentally pass a closure with the wrong signature or forget to handle the `Close` frame. The returned `axum::response::Response` is the HTTP 101 Switching Protocols response that Axum sends back to the client.

Usage in a route handler:

```rust
async fn ws_endpoint(
    upgrade: WebSocketUpgrade,
    handler: Resolved<MyHandler>,
) -> axum::response::Response {
    websocket(upgrade, handler.into_inner())
}
```

The handler is resolved from the DI container via `Resolved<T>`, extracted from the `Arc`, and passed to `websocket()`. The `on_upgrade` callback spawns implicitly inside Axum — you don't manage the task yourself.

---

## SSE: bridging server events to an HTTP response body

Server-Sent Events are simpler than WebSockets: a single HTTP response with `Content-Type: text/event-stream` that the server writes to and the browser reads from. The challenge is bridging an arbitrary server-side event source to the Axum response body stream.

`sse_channel()` at line 32 solves this:

```rust
pub fn sse_channel(capacity: usize) -> (mpsc::Sender<Event>, Sse<SseStream>) {
    let (sender, receiver) = mpsc::channel(capacity.max(1));
    let stream = futures_util::stream::unfold(receiver, |mut receiver| async move {
        receiver.recv().await.map(|event| (Ok(event), receiver))
    });
    (sender, Sse::new(Box::pin(stream)))
}

pub type SseStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>;
```

The function creates a bounded `mpsc::channel<Event>` and returns both ends. The `sender` half is kept by the server-side code — a background task, a service method, or an event listener. It calls `sender.send(Event::default().data("progress: 42%")).await` whenever there's new data to push.

The `receiver` half is converted into a `Stream` via `stream::unfold`, which turns the async `recv()` call into a lazy stream item. `Sse::new()` wraps this in an Axum SSE response, setting the correct headers automatically. The returned `Sse<SseStream>` can be returned directly from an Axum route handler.

The `capacity.max(1)` guard ensures the channel is never zero-sized — a zero-capacity channel would deadlock immediately because `send()` requires a matching `recv()` that hasn't been set up yet.

The `SseStream` type alias uses `Infallible` as the error type because the `mpsc` receiver never errors — `recv()` returns `None` when all senders are dropped, which `stream::unfold` maps to stream termination. There's no error path to propagate to the HTTP layer.

---

## When to use which

**Use `#[web_socket_gateway]` when:**
- You have multiple named events that map to handler methods (`chat.message`, `user.typing`, `room.join`)
- You need room-based broadcasting with `WsConnections`
- You want the `parse_incoming()` message format contract enforced for you
- The gateway fits the singleton-in-DI-container model

**Use native `websocket()` / `WebSocketHandler` when:**
- You need custom handshake logic (auth tokens in query parameters, sub-protocol negotiation)
- The connection lifecycle doesn't fit the gateway model (one-shot RPC over WebSocket, binary frames)
- You're writing a raw Axum route via `configure_router()` and can't use the macro system
- You need to integrate with a third-party WebSocket protocol that has its own framing

**Use `sse_channel()` when:**
- You need a one-directional event stream (progress updates, log tailing, notification feed)
- The client is a browser that supports `EventSource` but not raw WebSocket
- You want to push events from multiple server-side sources into a single response stream
- You're integrating with an existing event system that already uses channels

---

## The architecture insight

These helpers live in `ironic-services`, not `ironic-platform` or `ironic-platform-axum`. They're categorized as **application services** — optional utilities that depend on Axum but are available to any Ironic application using the Axum platform adapter. They don't extend the framework traits or modify the pipeline. They're just Rust functions that take Axum types and return Axum types, with Ironic's type constraints layered on top.

This is the right boundary. The `#[web_socket_gateway]` macro generates code that integrates with the DI container, route registration, and lifecycle hooks. The native helpers don't — they're direct-to-Axum, used inside `configure_router()` closures or raw route handlers. Choose the right abstraction for the job, and escape when you need to.
