---
title: OnRequestDestroy
description: Per-request cleanup — close resources, flush logs, record metrics
---

# OnRequestDestroy

`OnRequestDestroy` fires **after the response is sent**, during request cleanup.

## Position

```
Controller Handler
  │
  ▼
OnRequestDestroy ◀── YOU ARE HERE
  │
  ▼
Request complete
```

## Trait

```rust
pub trait OnRequestDestroy {
    async fn on_request_destroy(&self);
}
```

## Basic Usage

```rust
pub struct RequestCleanup;

impl OnRequestDestroy for RequestCleanup {
    async fn on_request_destroy(&self) {
        tracing::debug!("request resources cleaned up");
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_request_destroy = [RequestCleanup],
)]
pub struct AppModule;
```

## Common Uses

| Use Case | Description |
|----------|-------------|
| Close DB transactions | Rollback/commit per-request transactions |
| Flush log buffers | Ensure all log entries are written |
| Record request duration | Log elapsed time from OnRequestInit timer |
| Release per-request memory | Clean up request-scoped allocations |
