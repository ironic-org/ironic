---
title: AfterShutdown
description: Final cleanup hook — runs after all modules destroyed and application shutdown complete.
---

# AfterShutdown

Runs after **all** `OnModuleDestroy` and `OnApplicationShutdown` callbacks have completed. This is the absolute last hook before the process exits.

## Use cases

- Final metrics flush (last resort)
- Logging shutdown duration and summary
- Cleaning up OS-level resources (temp files, named pipes)
- Sending final heartbeat to monitoring

## Signature

```rust
pub trait AfterShutdown: Send + Sync + 'static {
    fn after_shutdown(&self) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{AfterShutdown, LifecycleError};

struct ShutdownLogger {
    start_time: std::time::Instant,
}

impl AfterShutdown for ShutdownLogger {
    async fn after_shutdown(&self) -> Result<(), LifecycleError> {
        let elapsed = self.start_time.elapsed();
        tracing::info!("Application shut down in {:?}", elapsed);
        Ok(())
    }
}
```

## When it runs

```
OnApplicationShutdown  -->  AfterShutdown  -->  [Process exits]
```

## Registration

```rust
ModuleDefinition::builder::<ShutdownLogger>()
    .after_shutdown()
    .build()
```

## Complete shutdown sequence

```
Shutdown signal received
    |
    v
BeforeShutdown              -- drain connections
    |
    v
OnModuleDestroy (reverse)  -- release resources
    |
    v
OnApplicationShutdown       -- final flush
    |
    v
AfterShutdown               -- last cleanup
    |
    v
Process exits
```

## Best practices

- Keep it **absolutely minimal** — the process is about to exit
- Don't rely on network resources (they may already be down)
- Log shutdown summary here for debugging
- Avoid allocations — GC/tokio may not run
