---
title: BeforeShutdown
description: Hook that fires immediately when a shutdown signal is received — before the server stops accepting connections.
---

# BeforeShutdown

Runs immediately after a shutdown signal is received, **before** the server stops accepting new connections.

## Use cases

- Draining in-flight connections gracefully
- Rejecting new requests with a friendly response
- Signalling load balancers to stop routing traffic
- Starting a grace period timer

## Signature

```rust
pub trait BeforeShutdown: Send + Sync + 'static {
    fn before_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{BeforeShutdown, LifecycleError, ShutdownSignal};

struct LoadBalancerDrainer;

impl BeforeShutdown for LoadBalancerDrainer {
    async fn before_shutdown(&self, signal: ShutdownSignal) -> Result<(), LifecycleError> {
        // Mark health endpoint as unhealthy
        set_health(false);
        // Wait for load balancer to detect
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok(())
    }
}
```

## When it runs

```
Shutdown signal (Ctrl+C / SIGTERM)
    |
    v
BeforeShutdown  -->  [Server stops accepting]  -->  OnModuleDestroy
```

## Registration

```rust
ModuleDefinition::builder::<LoadBalancerDrainer>()
    .before_shutdown()
    .build()
```

## Drain pattern

```rust
impl BeforeShutdown for ConnectionDrainer {
    async fn before_shutdown(&self, _signal: ShutdownSignal) -> Result<(), LifecycleError> {
        self.health.set_ready(false);
        // Give in-flight requests time to complete
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && self.active_connections() > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }
}
```

## Best practices

- Keep it **fast** — the shutdown timeout is ticking
- Use `BeforeShutdown` for pre-shutdown draining
- Use `AfterShutdown` for final cleanup after everything is destroyed
