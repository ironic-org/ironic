---
title: Events
description: Use Ironic's in-process event bus for simple application events.
---

# Events

Ironic events are local to one application process. Use them when one part of
your application should react to an action without tightly coupling to the code
that caused it.

Enable the feature:

```toml
ironic = { version = "1.0", features = ["events"] }
```

```rust
use ironic::services::events::EventBus;

let events = EventBus::default();
events.emit(UserCreated { id: 42 }).await;
```

For queues, brokers, and cross-service messaging, choose the library that fits
your application and register it as a normal Rust provider.
