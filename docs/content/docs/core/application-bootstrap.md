---
title: Application Bootstrap
description: Run code after ALL modules are initialized but before the server starts listening — background tasks, final validation, cache warmup, and more.
---

# OnApplicationBootstrap

## What is it?

`OnApplicationBootstrap` is a lifecycle hook that fires after every module in your application has been initialized, but **before** the HTTP server starts accepting requests. It's the *last chance* to run code before your app goes live.

Think of it like the final checklist before opening a store:
- **OnModuleInit** — Each department sets up their area (database connects, cache initializes)
- **OnApplicationBootstrap** — The manager walks through the store, verifies everything works together, flips the "Open" sign
- **Server starts listening** — Customers come in

## The trait

```rust
use ironic::LifecycleFuture;

pub trait OnApplicationBootstrap: Send + Sync + 'static {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_>;
}
```

`LifecycleFuture<'_>` is a type alias for `Pin<Box<dyn Future<Output = Result<(), LifecycleError>> + Send + '_>>`. In practice, you always write it like this:

```rust
impl OnApplicationBootstrap for MyService {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            // your code here
            Ok(())
        })
    }
}
```

## When to use it

| Scenario | Use OnApplicationBootstrap? |
|---|---|
| Start a background task that needs other services | **Yes** — all modules are ready, DI is fully wired |
| Run a health check across all modules | **Yes** — every provider is initialized |
| Schedule a cron job | **Yes** — the app is up, dependencies resolved |
| Warm an external cache | **Yes** — DB/cache modules are ready |
| Seed test data | OnModuleInit (module-level) or OnApplicationBootstrap (app-level) |
| Open a database connection | OnModuleInit — per-module resource setup |

**Rule of thumb:** If your startup code depends on services from *other* modules, use `OnApplicationBootstrap`. If it only depends on your own module's resources, use `OnModuleInit`.

## Full example — background cron task

The canonical real-world example from the blog-api:

```rust
use std::sync::Arc;
use ironic::{Injectable, LifecycleFuture, OnApplicationBootstrap};
use ironic::services::scheduling;

#[derive(Injectable)]
pub struct StatsReporter {
    service: Arc<BlogService>,
}

impl OnApplicationBootstrap for StatsReporter {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        let svc = Arc::clone(&self.service);
        Box::pin(async move {
            let _task = scheduling::cron("0 * * * * *", move || {
                let svc = Arc::clone(&svc);
                async move {
                    match svc.stats() {
                        Ok(s) => tracing::info!(total = s.total, "hourly stats"),
                        Err(e) => tracing::error!(error = %e, "stats failed"),
                    }
                }
            });

            tracing::info!("stats reporter started");
            Ok(())
        })
    }
}
```

**What's happening:**
1. `StatsReporter` is a DI provider that depends on `BlogService`
2. After ALL modules finish `OnModuleInit`, the framework calls `on_application_bootstrap`
3. The callback spawns a cron task that runs every hour
4. It returns `Ok(())` — the framework then proceeds to start the HTTP server

**The `Arc::clone` pattern:** Since the cron task runs in a separate async context, it needs its own `Arc<BlogService>` to outlive the bootstrap callback. The bootstrap function itself is short-lived — it just *schedules* work.

## Another example — cache warmup

```rust
use std::sync::Arc;
use ironic::{Injectable, LifecycleFuture, OnApplicationBootstrap};

#[derive(Injectable)]
pub struct CacheWarmer {
    cache: Arc<RedisCache>,
    product_service: Arc<ProductService>,
}

impl OnApplicationBootstrap for CacheWarmer {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        let cache = Arc::clone(&self.cache);
        let products = self.product_service.list_all().unwrap_or_default();
        Box::pin(async move {
            for product in products {
                cache.set_json(
                    &format!("product:{}", product.id),
                    &product,
                    Some(Duration::from_secs(3600)),
                ).await.ok();
            }
            tracing::info!("warmed {} products in cache", products.len());
            Ok(())
        })
    }
}
```

## Registration

Lifecycle hooks are registered on a `ModuleDefinition` via `LifecycleDefinition::builder()`:

```rust
#[derive(Module)]
#[module(
    imports = [BlogsModule],
    providers = [StatsReporter],
)]
pub struct TasksModule;

// The lifecycle hook is automatically picked up if StatsReporter
// implements OnApplicationBootstrap and is registered as a provider.
```

**Important:** Lifecycle hooks are discovered automatically when a provider implements one of the four lifecycle traits and is registered via `providers` in a `#[module(...)]` attribute. No manual `LifecycleDefinition::builder()` needed for the common case.

## Execution order

```
Bottom-up (reverse dependency order):
  1. Deepest module: OnModuleInit     ← DB connects
  2. Middle module: OnModuleInit      ← depends on DB
  3. Root module: OnModuleInit        ← depends on everything
  ── All modules initialized ──
  4. Deepest module: OnApplicationBootstrap  ← DB checks
  5. Middle module: OnApplicationBootstrap   ← cache warmup
  6. Root module: OnApplicationBootstrap     ← cron tasks
  ── Server starts ──
```

The framework topologically sorts module imports. Modules that are imported by others (leaves) initialize first. Root modules (the application module) initialize last. `OnApplicationBootstrap` follows the same forward order.

## Error handling

If a bootstrap callback returns `Err(LifecycleError)`, the application fails to start. No server starts. The framework runs `OnModuleDestroy` in **reverse order** for all modules that successfully initialized — no resources are leaked.

```rust
// If this fails, the server never starts
fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
    Box::pin(async move {
        let health = check_external_api().await?; // ← if this fails, app shuts down
        Ok(())
    })
}
```

## `OnApplicationBootstrap` vs `OnModuleInit`

| | OnModuleInit | OnApplicationBootstrap |
|---|---|---|
| **When** | After YOUR module's deps resolve | After ALL modules finish `OnModuleInit` |
| **Scope** | Per-module | App-wide |
| **Dependencies** | Your module's imported modules | Every module in the graph |
| **Best for** | DB migrations, connection pools | Background tasks, cross-module checks, scheduling |
| **On failure** | Your module's destroy runs | ALL modules' destroy runs in reverse |

## `OnApplicationBootstrap` vs `OnApplicationShutdown`

| | OnApplicationBootstrap | OnApplicationShutdown |
|---|---|---|
| **Direction** | Startup (forward) | Shutdown (reverse) |
| **Purpose** | Initialize cross-module work | Clean up cross-module work |
| **Signal** | None | Receives `ShutdownSignal` (Interrupt, Terminate, Custom) |
| **Best for** | Starting tasks | Gracefully stopping tasks, flushing metrics |

## Common mistakes

| Mistake | Fix |
|---|---|
| Starting a task without `Arc::clone` | The task runs in a separate async scope — it needs its own `Arc` |
| Depending on a service not in `imports` | Bootstrap runs after all deps — add missing modules to `imports` |
| Blocking the event loop in bootstrap | Use `tokio::spawn` for long-running tasks, return `Ok(())` quickly |
| Panicking in bootstrap | Return `Err(LifecycleError::new(...))` instead — allows clean rollback |
| Using `async fn` directly | The trait requires `LifecycleFuture` — use `Box::pin(async move { ... })` |

## What you learned

- [x] `OnApplicationBootstrap` runs after every module's `OnModuleInit` completes
- [x] Use it for cross-module startup: background tasks, scheduling, cache warmup
- [x] Return `Ok(())` to proceed to server listen, `Err(...)` to abort startup
- [x] Use `Arc::clone` for data needed by spawned tasks
- [x] Order: forward (bottom-up) on init, reverse on shutdown
- [x] Registration is automatic when the provider implements the trait
