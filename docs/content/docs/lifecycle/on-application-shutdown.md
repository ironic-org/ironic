---
title: OnApplicationShutdown
description: After all modules destroyed — final cleanup, metrics flush
---

# OnApplicationShutdown

`OnApplicationShutdown` fires **after all modules** have been destroyed.
This is the second-to-last hook before the process exits.

## Position

```
OnModuleDestroy (all modules)
  │
  ▼
OnApplicationShutdown ◀── YOU ARE HERE
  │
  ▼
AfterShutdown
  │
  ▼
Process exits
```

## Trait

```rust
pub trait OnApplicationShutdown {
    async fn on_application_shutdown(&self);
}
```

## Basic Usage

```rust
pub struct MetricsFlusher;

impl OnApplicationShutdown for MetricsFlusher {
    async fn on_application_shutdown(&self) {
        tracing::info!("flushing metrics before exit");
        metrics::flush();
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_shutdown = [MetricsFlusher],
)]
pub struct AppModule;
```
