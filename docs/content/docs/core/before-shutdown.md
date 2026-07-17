---
title: BeforeShutdown
description: Runs BEFORE server stops accepting connections — drain connections, signal load balancers, reject new requests gracefully.
---

# BeforeShutdown

## What it does

`BeforeShutdown` runs immediately after a shutdown signal (SIGTERM, Ctrl-C) is received, but **before** the HTTP server stops accepting connections. This is the window for:

- Draining in-flight connections
- Signaling your load balancer to stop routing traffic
- Rejecting new requests with a 503 "shutting down" response
- Logging the shutdown event

## How to use

```rust
use ironic::{BeforeShutdown, LifecycleFuture, ShutdownSignal};
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
            tracing::info!(?signal, "server draining");

            // Wait for in-flight requests to complete
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            Ok(())
        })
    }
}
```

## Execution order

```
SIGTERM received
    │
    ▼
BeforeShutdown  ← DRAIN HERE (server still accepting)
    │
    ▼
Server stops accepting connections
    │
    ▼
OnApplicationShutdown
OnModuleDestroy
AfterShutdown
```

## Middleware for soft rejection

```rust
impl Middleware for DrainMiddleware {
    fn handle(&self, ctx: &mut RequestContext, next: MiddlewareNext) -> PipelineFuture {
        if self.flag.is_draining() {
            return Box::pin(async {
                Err(HttpError::service_unavailable("DRAINING", "server shutting down"))
            });
        }
        next.run(ctx)
    }
}
```
