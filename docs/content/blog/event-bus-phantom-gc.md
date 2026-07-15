---
title: "Typed Event Bus — phantom subscription filtering and channel garbage collection"
description: "How Ironic's in-process event bus uses PhantomData vigilantes, bounded mpsc channels, and publish-time GC to deliver typed events with automatic backpressure and zero subscriber leak."
date: "2026-07-15"
author: "Ironic Team"
---

# Typed Event Bus — phantom subscription filtering and channel garbage collection

In-process event buses in Rust face two problems that are invisible in garbage-collected runtimes: how do you filter events per subscriber type without runtime type checks at the publish site, and how do you clean up dead subscribers when their receivers are dropped? Ironic's `EventBus` solves both with a compact 79-line implementation built on `TypeId`, `PhantomData`, and Tokio's bounded `mpsc`.

## The storage model: `TypeId` → `Vec<Sender>`

The `EventBus` is a `Clone + Default` struct wrapping a single shared container:

```rust
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<TypeId, Vec<mpsc::Sender<ErasedEvent>>>>>,
}
```

Events flow through the bus as `Arc<dyn Any + Send + Sync>`. Every published event is instantaneously wrapped in `Arc`, which means the same allocation can be cloned to every subscriber channel without duplicating the payload. The outer `RwLock` allows concurrent reads (publishing to multiple subscribers) while serializing writes (subscription registration and garbage collection). The `TypeId` key partitions subscribers by the event type they declared interest in — so a subscriber registered for `OrderPlaced` will never see `PaymentProcessed` events appear in its channel.

## The phantom subscription

The `EventSubscription<E>` type is where the type-system trickery lives:

```rust
pub struct EventSubscription<E> {
    receiver: mpsc::Receiver<ErasedEvent>,
    marker: std::marker::PhantomData<fn() -> E>,
}
```

The `receiver` field is a Tokio `mpsc::Receiver` carrying opaque `ErasedEvent` values. The `marker` field is `PhantomData<fn() -> E>` — it occupies zero bytes at runtime but informs the compiler what concrete type this subscription represents. Critically, `PhantomData<fn() -> E>` makes `EventSubscription<E>` invariant over `E` (because `fn() -> E` is invariant in its return type). This prevents any accidental coercion between subscription types and ensures the `recv()` method can safely call `downcast::<E>()` with a known type.

The `marker` never participates in any runtime operation. It exists purely so that `EventSubscription<OrderPlaced>` and `EventSubscription<PaymentReceived>` are distinct, incompatible types at compile time.

## Subscribe: bounded channels with explicit capacity

The `subscribe()` method creates a per-subscriber channel:

```rust
pub async fn subscribe<E: Send + Sync + 'static>(
    &self,
    capacity: usize,
) -> EventSubscription<E> {
    let (sender, receiver) = mpsc::channel(capacity.max(1));
    self.subscribers
        .write()
        .await
        .entry(TypeId::of::<E>())
        .or_default()
        .push(sender);
    EventSubscription { receiver, marker: std::marker::PhantomData }
}
```

The caller must specify a `capacity` — a deliberate design choice that forces the developer to think about backpressure. A `capacity` of 64 means the bus will accept up to 64 events for this subscriber before `sender.send().await` blocks the publisher. The `.max(1)` guard prevents degenerate zero-capacity channels. The sender is pushed into the `Vec` for `E`'s `TypeId` entry, and the subscriber walks away with the `receiver` half.

## `recv()`: phantom filtering in a loop

Here is where the phantom type earns its keep:

```rust
pub async fn recv(&mut self) -> Option<Arc<E>> {
    while let Some(event) = self.receiver.recv().await {
        if let Ok(event) = event.downcast::<E>() {
            return Some(event);
        }
    }
    None
}
```

The `recv()` method sits in a tight `while` loop. Each iteration pulls an `Arc<dyn Any + Send + Sync>` from the channel, attempts `downcast::<E>()`, and either returns the successfully cast event or silently discards the mismatch and tries again. The loop terminates (returning `None`) only when the channel is closed — meaning all senders have been dropped.

Why is this loop necessary? Because the channel carries `ErasedEvent` — `Arc<dyn Any + Send + Sync>` — not `Arc<E>`. A subscriber for `OrderPlaced` might theoretically receive an `Arc<PaymentProcessed>` if a bug in the publish-side routing sent an event to the wrong `TypeId` bucket. The `downcast` loop is a defense-in-depth measure: it silently filters out misrouted events so a subscriber only ever sees its declared type.

In practice, the routing is correct because `publish()` only sends to the `TypeId` entry matching the published event type. The `downcast` loop is a guardrail, not a normal codepath — analogous to the `TypeMismatch` downcasts in the CQRS bus.

## Garbage collection at publish time

The most elegant trick in the bus is the garbage collector hidden inside `publish()`:

```rust
self.subscribers
    .write()
    .await
    .entry(TypeId::of::<E>())
    .or_default()
    .retain(|sender| !sender.is_closed());
```

After every publish, the bus acquires the write lock and calls `retain()` on the subscriber list for the event type. `sender.is_closed()` returns `true` when the corresponding `receiver` has been dropped — meaning the subscriber is gone. The `retain()` call sweeps these dead senders out of the vector.

This design avoids an explicit `unsubscribe()` method entirely. When a subscriber wants to stop receiving events, it simply drops its `EventSubscription<E>`. The `receiver` is dropped, the `sender` becomes closed, and the next `publish()` call removes the sender from the active list. The GC is amortized — each publish call does O(n) work proportional to the number of registered subscribers for that type. For typical applications with tens of subscribers per event type, this is negligible.

## Why bounded channels?

The choice of bounded `mpsc::channel(capacity)` over unbounded channels is deliberate. Unbounded channels can accumulate events faster than a subscriber can process them, leading to unbounded memory growth under load. Bounded channels create automatic backpressure: if a subscriber falls behind and its channel fills up, the publish call blocks until the subscriber catches up or its receiver is dropped (triggering GC on the next iteration).

This is a critical property for production systems. A slow email service subscriber should not cause the event bus to allocate gigabytes of buffered events. With bounded channels, the slow subscriber either keeps up, or the system degrades gracefully as `publish()` applies backpressure to the publisher, which in turn slows the HTTP handler, which in turn applies backpressure to the client.

## A concrete example

```rust
let bus = EventBus::default();

let mut email = bus.subscribe::<OrderPlaced>(64);
let mut analytics = bus.subscribe::<OrderPlaced>(32);
let mut inventory = bus.subscribe::<OrderPlaced>(128);

tokio::spawn(async move {
    while let Some(order) = email.recv().await {
        send_confirmation_email(&order).await;
    }
});

bus.publish(OrderPlaced { order_id: 1, user_id: 42 }).await;
```

Three subscribers for the same event type, each with a different channel capacity reflecting their anticipated processing speed. The email service gets 64 slots, analytics gets 32 (it can afford to drop early samples), and inventory gets 128 (it must never miss an event). When `email`'s task finishes and its `EventSubscription` is dropped, the next `publish(OrderPlaced{...})` call removes its sender from the subscriber list — no explicit cleanup required.

The entire system: compile-time type safety via `PhantomData`, automatic backpressure via bounded channels, and automatic GC via `retain()`. All in 79 lines of Rust.
