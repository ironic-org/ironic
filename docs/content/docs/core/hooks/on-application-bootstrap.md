---
title: OnApplicationBootstrap
description: Cross-module startup — background tasks, cron jobs, health checks. Runs after ALL modules finish OnModuleInit.
---

# OnApplicationBootstrap

## What is it?

`OnApplicationBootstrap` fires after EVERY module has completed its `OnModuleInit`. This is when you run cross-module startup tasks — the things that need ALL services to be ready.

**Analogy:** After every department sets up (OnModuleInit), the manager does a final walkthrough before opening the store. That's OnApplicationBootstrap — the last check before customers arrive.

## When it fires

```
Module A: OnModuleInit → ✅
Module B: OnModuleInit → ✅
Module C: OnModuleInit → ✅
─────────────────────────
OnApplicationBootstrap   ← YOU ARE HERE (all modules ready)
─────────────────────────
OnServerReady
Server listens
```

## Why you need it

`OnModuleInit` is per-module. You can't start a cron job there because other modules might not be initialized yet. `OnApplicationBootstrap` guarantees everything is ready.

**Without it:**
```rust
// ❌ Wrong — StatsReporter might not have BlogService ready yet
impl OnModuleInit for StatsReporter {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        self.blog_service.list_posts()?; // ← BlogService might not be initialized!
    }
}
```

**With it:**
```rust
// ✅ Correct — BlogService is guaranteed initialized
impl OnApplicationBootstrap for StatsReporter {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        self.blog_service.list_posts()?; // ← Safe! All modules are ready.
    }
}
```

## How to use

```rust
use std::sync::Arc;
use ironic::{Injectable, LifecycleFuture, OnApplicationBootstrap};

#[derive(Injectable)]
pub struct StatsReporter {
    blog_service: Arc<BlogService>,
}

impl OnApplicationBootstrap for StatsReporter {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        let svc = Arc::clone(&self.blog_service);
        Box::pin(async move {
            // Start a cron job that queries BlogService every hour
            let _task = scheduling::cron("0 * * * * *", move || {
                let svc = Arc::clone(&svc);
                async move {
                    match svc.stats() {
                        Ok(s) => tracing::info!(total = s.total, "hourly stats"),
                        Err(e) => tracing::error!(error = %e, "stats failed"),
                    }
                }
            });
            tracing::info!("Stats reporter started");
            Ok(())
        })
    }
}
```

**The `Arc::clone` pattern:** The cron task runs in a separate async context. It needs its own `Arc<BlogService>` to outlive the bootstrap callback. The bootstrap function itself is short-lived — it just SCHEDULES work.

## Common patterns

### Self health check

```rust
impl OnApplicationBootstrap for HealthChecker {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            // Verify all critical services are reachable
            self.db.ping().await?;
            self.redis.ping().await?;
            self.cache.stats().await?;
            tracing::info!("All services healthy");
            Ok(())
        })
    }
}
```

### Feature flag gating

```rust
impl OnApplicationBootstrap for FeatureToggler {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let enabled = self.config.features.enable_ai;
            tracing::info!(enabled, "AI feature status");
            Ok(())
        })
    }
}
```

### Warm external caches

```rust
impl OnApplicationBootstrap for CacheWarmer {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let count = self.cache.warm_popular_products().await?;
            tracing::info!("Warmed {} popular products", count);
            Ok(())
        })
    }
}
```

## Registration

```rust
#[derive(Module)]
#[module(
    imports = [BlogsModule],
    providers = [StatsReporter],
    lifecycle_bootstrap = [StatsReporter],  // ← OnApplicationBootstrap
)]
pub struct TasksModule;
```

## Error handling

If `OnApplicationBootstrap` returns `Err(...)`, the application **aborts startup**. ALL modules that successfully completed `OnModuleInit` get their `OnModuleDestroy` called in reverse order.

## OnApplicationBootstrap vs OnServerReady

| | OnApplicationBootstrap | OnServerReady |
|---|---|---|
| **Server state** | Not built yet | Built, not listening |
| **HTTP calls** | Can't call own endpoints | Can call health checks |
| **Modules** | All initialized | All initialized + platform built |
| **Best for** | Background tasks, cron | Readiness checks, orchestrator notification |
