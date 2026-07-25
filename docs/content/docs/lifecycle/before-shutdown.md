---
title: BeforeShutdown
description: Graceful shutdown — drain connections, notify load balancers, prepare to stop
---

# BeforeShutdown

`BeforeShutdown` fires when a **shutdown signal is received**. The server
stops accepting new connections, but in-flight requests continue.

## Position

```
Shutdown Signal Received
  │
  ▼
BeforeShutdown ◀── YOU ARE HERE (stop accepting new connections)
  │
  ▼
Drain in-flight requests
  │
  ▼
OnModuleDestroy
```

## Trait

```rust
pub trait BeforeShutdown {
    async fn before_shutdown(&self, signal: ShutdownSignal);
}
```

## Basic Usage

```rust
pub struct ConnectionDrainer {
    pool: Arc<DatabasePool>,
}

impl BeforeShutdown for ConnectionDrainer {
    async fn before_shutdown(&self, signal: ShutdownSignal) {
        tracing::info!("shutting down — draining connections");

        // Graceful drain with timeout
        tokio::select! {
            _ = self.pool.drain() => {
                tracing::info!("connections drained");
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                tracing::warn!("drain timed out — forcing shutdown");
            }
        }
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_before_shutdown = [ConnectionDrainer],
)]
pub struct AppModule;
```

## Common Uses

- Drain database connection pools
- Notify load balancers to remove this instance
- Complete in-flight request processing
- Flush metrics and tracing data
- Close external service connections
