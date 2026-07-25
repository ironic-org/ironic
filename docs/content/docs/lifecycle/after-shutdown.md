---
title: AfterShutdown
description: Final hook — guaranteed to run even if other shutdown hooks fail
---

# AfterShutdown

`AfterShutdown` is the **LAST hook** to fire during the lifecycle.
It is guaranteed to run even if previous shutdown hooks failed.

## Position

```
OnApplicationShutdown
  │
  ▼
AfterShutdown ◀── YOU ARE HERE (guaranteed to fire)
  │
  ▼
Process exits
```

## Trait

```rust
pub trait AfterShutdown {
    async fn after_shutdown(&self);
}
```

## Basic Usage

```rust
pub struct FinalFlusher;

impl AfterShutdown for FinalFlusher {
    async fn after_shutdown(&self) {
        // Guaranteed to run — even if other shutdown hooks fail
        tracing::info!("goodbye from Ironic!");
        metrics::flush();
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_after_shutdown = [FinalFlusher],
)]
pub struct AppModule;
```

## Guarantee

Unlike other shutdown hooks, `AfterShutdown` **always runs**:

```
BeforeShutdown → may fail → logged
OnModuleDestroy → may fail → logged but other destroys still run
OnApplicationShutdown → may fail → logged
AfterShutdown → ALWAYS RUNS ✓
```
