---
title: Dead Letter Queue
description: Capture undelivered events for diagnostics — see what events were lost and why.
---

# Dead Letter Queue

## What is it?

When an event subscriber's buffer is full or the receiver is dropped, events are lost. The dead-letter queue captures these undelivered events so you can inspect what was missed.

## How to use

```rust
use ironic::services::events::EventBus;

let bus = EventBus::default();

bus.publish("user.created".to_string()).await;

let undelivered = bus.drain_dead_letters().await;
for letter in &undelivered {
    tracing::warn!(
        type_name = letter.type_name,
        event = letter.event,
        "event was not delivered"
    );
}
```

## DeadLetter structure

```rust
pub struct DeadLetter {
    pub type_name: &'static str,  // Rust type name of the event
    pub event: String,            // Debug string of the event data
}
```

## When to use

| Scenario | Action |
|----------|--------|
| Subscriber crashed | Inspect dead letters to replay missed events |
| Buffer overflow | Increase buffer size or add more subscribers |
| Development debugging | See which events are being published |

## What you learned

- [x] `EventBus` captures undelivered events in a dead-letter queue
- [x] `drain_dead_letters()` returns and clears all undelivered events
- [x] Each entry has the type name and debug representation
