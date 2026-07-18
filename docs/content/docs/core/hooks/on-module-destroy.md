---
title: OnModuleDestroy
description: Per-module cleanup — close connections, flush buffers, release resources. Runs in reverse order on shutdown.
---

# OnModuleDestroy

## What is it?

`OnModuleDestroy` is the cleanup counterpart of `OnModuleInit`. It runs when the application shuts down — in the **reverse order** of initialization.

**Analogy:** At closing time, the electronics department powers down displays BEFORE the bakery turns off ovens. Why? Because customers already left. Cleanup order is the reverse of setup order.

## When it fires

```
Shutdown signal received
    │
BeforeShutdown → OnApplicationShutdown → OnModuleDestroy
                                              │
                                    Reverse topological order
                                              │
                                    Module C: destroy first (initialized last)
                                    Module B: destroy
                                    Module A: destroy last (initialized first)
                                              │
                                    AfterShutdown
```

## Why you need it

Without `OnModuleDestroy`, cleanup is manual and error-prone:

```rust
// ❌ Manual cleanup — easy to miss, wrong order
app.listen("0.0.0.0:3000").await?;
database.close().await;   // ← what if server crashed?
cache.flush().await;      // ← wrong order?
metrics.flush().await;    // ← never called
```

With `OnModuleDestroy`, each module cleans itself up in the correct order:

```rust
// ✅ Automatic — framework handles order
#[module(
    providers = [Database, Cache, Metrics],
    lifecycle_destroy = [Database, Cache, Metrics],
)]
```

## How to use

```rust
impl OnModuleDestroy for Database {
    fn on_module_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            tracing::info!("Closing database pool...");
            self.pool.close().await;
            tracing::info!("Database pool closed");
            Ok(())
        })
    }
}
```

## Execution order guarantee

Modules are destroyed in **reverse topological order**:

```
Startup:  DatabaseModule → UsersModule → AppModule (forward)
Shutdown: AppModule → UsersModule → DatabaseModule (reverse)
```

This means `AppModule` (which depends on everything) is destroyed FIRST, and `DatabaseModule` (the foundation) is destroyed LAST. No dependency is destroyed while someone still needs it.

## Error handling

`OnModuleDestroy` is **best-effort** on shutdown. If one module's destroy fails, the remaining modules STILL get their destroy called. The first error is returned, but no resources are leaked.

```rust
impl OnModuleDestroy for Database {
    fn on_module_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            // Even if the pool is already closed, this succeeds
            let _ = self.pool.close().await;
            Ok(())
        })
    }
}
```

## Common patterns

### Close database pool

```rust
impl OnModuleDestroy for Database {
    fn on_module_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.pool.close().await;
            Ok(())
        })
    }
}
```

### Flush buffered writes

```rust
impl OnModuleDestroy for EventLogger {
    fn on_module_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.buffer.flush().await?;
            Ok(())
        })
    }
}
```

### Send shutdown telemetry

```rust
impl OnModuleDestroy for Telemetry {
    fn on_module_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.client.send_event("app.shutdown", &json!({
                "uptime_seconds": self.start_time.elapsed().as_secs(),
            })).await.ok();
            Ok(())
        })
    }
}
```
