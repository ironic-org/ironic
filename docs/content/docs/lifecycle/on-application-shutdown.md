---
title: OnApplicationShutdown
description: Last lifecycle hook before process exit — runs after all modules are destroyed.
---

# OnApplicationShutdown

Runs after all module destroy hooks have completed and before the process exits.

## Use cases

- Final metrics flush before exit
- Last-chance logging
- Notifying external systems of shutdown
- Cleaning up temporary files

## Signature

```rust
pub trait OnApplicationShutdown: Send + Sync + 'static {
    fn on_application_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnApplicationShutdown, LifecycleError, ShutdownSignal};

struct MetricsFlusher;

impl OnApplicationShutdown for MetricsFlusher {
    async fn on_application_shutdown(&self, signal: ShutdownSignal) -> Result<(), LifecycleError> {
        tracing::info!(?signal, "Shutting down, flushing metrics...");
        metrics::flush().await;
        Ok(())
    }
}
```

## When it runs

```
OnModuleDestroy  -->  OnApplicationShutdown  -->  AfterShutdown  -->  [Exit]
```

This runs **after** all modules have been destroyed.

## Registration

```rust
ModuleDefinition::builder::<MetricsFlusher>()
    .application_shutdown()
    .build()
```

## Shutdown signals

| Signal | Source |
|--------|--------|
| `ShutdownSignal::Interrupt` | SIGINT (Ctrl+C) |
| `ShutdownSignal::Terminate` | SIGTERM |
| `ShutdownSignal::Custom(msg)` | Application-defined |

## Best practices

- This is your **last chance** to flush data — make it count
- Use the `signal` parameter to react differently to different signals
- Keep it fast — the OS may kill the process after the timeout
- Don't start new work here — only finish existing work
