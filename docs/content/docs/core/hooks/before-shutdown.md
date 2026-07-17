---
title: BeforeShutdown
description: Runs when shutdown begins — drain connections, reject new requests, signal load balancers before the server stops.
---

# BeforeShutdown

Runs **immediately** after a shutdown signal is received, but **before** the server stops accepting new connections. This is the first shutdown hook.

## When it fires

```
SIGTERM / Ctrl-C
    │
    ▼
BeforeShutdown  ← YOU ARE HERE
    │
    ▼
Server stops accepting connections
    │
    ▼
OnApplicationShutdown
OnModuleDestroy
AfterShutdown
```

At this point, the server IS still accepting requests. You have a brief window to drain or reject gracefully.

## The trait

```rust
pub trait BeforeShutdown: Send + Sync + 'static {
    fn before_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why BeforeShutdown |
|---|---|
| Set a "draining" flag that all middleware checks | Graceful connection drain |
| Signal your load balancer to stop routing | HAProxy/AWS ALB deregistration |
| Cancel long-running WebSocket subscriptions | Prevent new messages during shutdown |
| Log shutdown start time | Shutdown duration benchmarking |

## Example — draining flag

```rust
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Injectable)]
pub struct DrainFlag {
    draining: AtomicBool,
}

impl DrainFlag {
    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }
}

impl BeforeShutdown for DrainFlag {
    fn before_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.draining.store(true, Ordering::SeqCst);
            tracing::info!(?signal, "server draining — rejecting new requests");

            // Give in-flight requests 2 seconds to complete
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Ok(())
        })
    }
}
```

In your middleware:

```rust
impl Middleware for DrainMiddleware {
    fn handle(&self, ctx: &mut RequestContext, next: MiddlewareNext) -> PipelineFuture {
        if self.flag.is_draining() {
            return Box::pin(async {
                Err(HttpError::service_unavailable("DRAINING", "server is shutting down"))
            });
        }
        next.run(ctx)
    }
}
```

## ShutdownSignal

The `signal` parameter tells you why the shutdown happened:

```rust
pub enum ShutdownSignal {
    Interrupt,            // Ctrl-C
    Terminate,            // SIGTERM (Kubernetes pod eviction)
    Custom(&'static str), // Programmatic shutdown
}
```
