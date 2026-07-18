---
title: OnApplicationShutdown
description: Application-level shutdown — runs after serving stops, before per-module destruction. Receives the shutdown signal.
---

# OnApplicationShutdown

## What is it?

`OnApplicationShutdown` fires after the HTTP server stops accepting connections but before individual modules are destroyed. It receives the `ShutdownSignal` so you know WHY the shutdown happened.

**Analogy:** The store manager announces "We're closing!" and everyone starts their closing checklist. `OnApplicationShutdown` is the manager's announcement — it tells everyone what's happening.

## When it fires

```
Shutdown signal received (SIGTERM / Ctrl-C)
    │
BeforeShutdown              ← Server STILL accepting
    │
Server stops accepting
    │
OnApplicationShutdown       ← YOU ARE HERE
    │
OnModuleDestroy (reverse)   ← Per-module cleanup
AfterShutdown
```

## The ShutdownSignal

```rust
pub enum ShutdownSignal {
    Interrupt,            // Ctrl-C
    Terminate,            // SIGTERM (Kubernetes pod eviction)
    Custom(&'static str), // Programmatic shutdown
}
```

## Why you need it

`OnModuleDestroy` runs per-module and doesn't know WHY the shutdown happened. `OnApplicationShutdown` gives you the signal — useful for differentiated behavior:

```rust
impl OnApplicationShutdown for GracefulShutdown {
    fn on_application_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_> {
        Box::pin(async move {
            match signal {
                ShutdownSignal::Terminate => {
                    // SIGTERM — we have ~30s. Do a quick flush.
                    self.metrics.flush_recent().await;
                }
                ShutdownSignal::Interrupt => {
                    // Ctrl-C — developer stopped it. Skip flush.
                    tracing::info!("Developer interrupt — skipping flush");
                }
                ShutdownSignal::Custom(reason) => {
                    tracing::warn!("Custom shutdown: {reason}");
                }
            }
            Ok(())
        })
    }
}
```

## How to use

```rust
impl OnApplicationShutdown for MetricsFlusher {
    fn on_application_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_> {
        Box::pin(async move {
            tracing::info!(?signal, "Application shutting down");
            self.push_to_collector().await.ok();
            Ok(())
        })
    }
}
```

## Registration

```rust
#[derive(Module)]
#[module(
    providers = [MetricsFlusher],
    lifecycle_shutdown = [MetricsFlusher],
)]
pub struct ObservabilityModule;
```

## Execution order

Runs in **reverse** registration order. If you registered `ModuleA` before `ModuleB`, ModuleB's `OnApplicationShutdown` runs first.

## Error handling

Best-effort — all hooks run even if some fail. Only the first error is returned.
