---
title: "WebSocket Connection Lifecycle — clients, rooms, and broadcasting"
description: "How Ironic's WsConnections manages connected clients with atomic IDs, room membership, and dual-lock broadcasting — plus the #[web_socket_gateway] code generation."
date: "2026-07-15"
author: "Ironic Team"
---

# WebSocket Connection Lifecycle — clients, rooms, and broadcasting

WebSocket infrastructure in most frameworks is a bag of callbacks glued to a raw socket. Ironic's `WsConnections` is a structured, lock-aware connection manager built on Tokio channels and `RwLock` — with atomic client IDs, room-based broadcasting, and a compile-time macro that generates the gateway boilerplate.

This post walks through the data structures and algorithms that power WebSocket connections in Ironic.

---

## The connection map: one channel per client

At `ws.rs:27-29`, the core state lives in two concurrent maps:

```rust
pub struct WsConnections {
    clients: Arc<RwLock<HashMap<ClientId, mpsc::UnboundedSender<WsMessage>>>>,
    rooms: Arc<RwLock<HashMap<String, Vec<ClientId>>>>,
}
```

Every connected client gets an `mpsc::UnboundedSender<WsMessage>` — an unbounded Tokio channel transmitter that pushes structured messages from the server to the client. The channel's receiver half lives in the WebSocket write-task spawned when the connection upgrades. When that task needs to send a frame to the browser, it reads from this receiver.

The `Arc<RwLock<>>` pattern means cloning `WsConnections` is cheap (one `Arc` increment), and multiple concurrent operations can read the map simultaneously. Writes — connections, disconnections, room joins — take an exclusive lock but are O(1) for the client map and O(rooms) for room membership scans.

`UnboundedSender` is chosen over a bounded channel because WebSocket backpressure is handled at the TCP layer — if the client can't keep up, the kernel buffer fills, the write task blocks, and the channel naturally drains. A bounded channel would introduce artificial frame drops that the application layer can't recover from.

---

## ClientId: monotonic atomics, no UUID

At `ws.rs:5-14`, client identity is a thin wrapper around a `u64`:

```rust
static NEXT_CLIENT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl ClientId {
    fn new() -> Self {
        Self(NEXT_CLIENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}
```

A single static `AtomicU64` with `fetch_add` under `Relaxed` ordering. This is the cheapest possible unique ID generator — no system call, no `Mutex`, no allocation, and no cache-line bouncing beyond the atomic itself. `Relaxed` is sufficient because the only correctness requirement is uniqueness; ordering relative to other atomic operations is irrelevant.

The counter starts at `1` (not `0`) so the zero value is never issued — useful as a sentinel for uninitialized fields. At 1 billion connections per second, the counter would overflow in roughly 584 years. It's effectively infinite.

---

## Rooms: Vec-based membership with join/leave

Rooms live in the second map: `HashMap<String, Vec<ClientId>>`. No `HashSet`, no `BTreeSet`, no linked structure — just a `Vec`.

`join_room()` at line 60 pushes the client ID onto the room's vector. `leave_room()` at line 66 does a linear `retain` to remove it. This is O(n) in room size, but real-world rooms (chat rooms, notification groups) rarely exceed a few thousand members. The simpler data structure avoids the memory overhead and pointer indirection of a `HashSet`, and the linear scan is cache-friendly.

The trade-off becomes visible at `disconnect()` on line 51:

```rust
pub async fn disconnect(&self, id: ClientId) {
    self.clients.write().await.remove(&id);
    let mut rooms = self.rooms.write().await;
    for members in rooms.values_mut() {
        members.retain(|c| *c != id);
    }
}
```

Disconnection removes the client from the client map (O(1)), then iterates **every room** to remove the client's ID. This is O(total members across all rooms), but it runs once per disconnection and avoids the alternative — storing a reverse index from `ClientId` to a list of room names — which would add allocation and synchronization overhead for the common case (join, broadcast, leave).

---

## Broadcasting: dual-lock read with per-recipient cloning

`broadcast_room()` at line 82 shows the two-lock pattern:

```rust
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
```

Two `read()` locks are acquired **sequentially**, not nested. This is intentional: each lock is held only for the duration of the lookup, and releasing the rooms lock before acquiring the clients lock prevents deadlock. The `members` vector is a snapshot — a client that disconnects between the two reads will be silently skipped (the `if let Some(sender)` branch is `None`).

The message is **cloned per recipient**. `WsMessage` contains two `String` fields (`event` and `data`), so cloning is a heap allocation per recipient. For large rooms, this is the broadcasting bottleneck. The alternative — `Arc<WsMessage>` and shared references — would require a different channel type (`broadcast` instead of `mpsc`), which changes the ordering and delivery semantics. Ironic chooses clone-per-recipient for simplicity, accepting that rooms with thousands of members should use a dedicated pub/sub system rather than in-process broadcasting.

`broadcast_all()` at line 74 follows the same pattern but skips the room lookup — it reads the client map, iterates all senders, and clones-per-recipient.

---

## Message format: `{"event": "...", "data": ...}`

`parse_incoming()` at line 128 enforces a structured message contract:

```rust
pub fn parse_incoming(text: &str) -> Result<IncomingMessage, String> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| format!("Invalid JSON: {e}"))?;
    let event = value.get("event")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing `event` field".to_string())?
        .to_string();
    let data = value.get("data")
        .map_or_else(|| "null".to_string(), ToString::to_string);
    Ok(IncomingMessage { event, data })
}
```

Every incoming WebSocket text frame must be valid JSON with a top-level `"event"` string field. The `"data"` field is optional — missing data becomes the JSON string `"null"`. This constraint is enforced at the **protocol boundary**, before any application handler sees the message. Malformed JSON or missing `event` returns an `Err(String)` that the gateway handler can convert into a structured error frame.

The `"data"` field is preserved as a raw JSON string — it's not deserialized into a typed struct at this layer. The gateway's event dispatch (in generated code) routes by `event` name and lets each handler deserialize `data` into its own expected type. This avoids double-deserialization and keeps the message parser fast.

---

## The `#[web_socket_gateway]` macro

At `ws_gateway.rs:6-33`, the macro receives a `LitStr` (the gateway path) and an `ItemStruct` (the gateway struct definition). It generates two associated functions:

```rust
impl #name {
    pub fn provider_definition() -> ::ironic::ProviderDefinition {
        ::ironic::ProviderDefinition::constructor(
            ::ironic::Scope::Singleton,
            vec![],
            |_resolver| async move { Ok(#name) },
        )
    }

    pub fn gateway_definition() -> ::ironic::WsGatewayDefinition {
        ::ironic::WsGatewayDefinition {
            path: #path.to_string(),
            controller: ::ironic::ProviderKey::of::<Self>(),
            handler_name: stringify!(#name),
        }
    }
}
```

`provider_definition()` generates a DI provider that constructs the gateway as a singleton — no dependencies, no factory arguments. The gateway struct itself must implement `Default` or be constructible with no arguments; the generated factory just returns `Ok(#name)`.

`gateway_definition()` builds a `WsGatewayDefinition` that maps the WebSocket path to the gateway's DI key and handler name. This definition is what `Application` registers during `build()` — the Axum adapter uses it to create an Axum WebSocket route at the given path, and the framework's startup logic resolves the gateway from the container before the first connection arrives.

The gateway struct you write contains event handler methods with `#[web_socket_event("event.name")]` attributes. Those are processed by a separate macro pass (on the `impl` block), not shown in this file — `ws_gateway.rs` only handles the struct-level annotation.

---

## Summary

Ironic's WebSocket layer is a lock-aware, channel-based connection manager with three key design choices. First, `ClientId` uses a monotonic atomic counter for zero-allocation uniqueness. Second, rooms use flat `Vec<ClientId>` membership with O(n) leave/disconnect — fast enough for typical room sizes and cache-friendly. Third, broadcasting takes sequential read locks on rooms and clients, cloning messages per recipient for simplicity over maximum throughput. The `#[web_socket_gateway]` macro generates the DI provider and gateway registration, keeping the struct you write focused on event handler logic.
