---
title: OnModuleInit
description: Per-provider initialization after dependencies are resolved.
---

# OnModuleInit

Runs after a provider's module and dependencies are fully resolved by the DI container.

## Use cases

- Opening database connections
- Loading cached data into memory
- Connecting to external services
- Starting background workers
- Warmup operations

## Signature

```rust
pub trait OnModuleInit: Send + Sync + 'static {
    fn on_module_init(&self) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnModuleInit, LifecycleError, Injectable};
use std::sync::Arc;

#[derive(Injectable)]
struct CacheService {
    redis: Arc<RedisPool>,
}

impl OnModuleInit for CacheService {
    async fn on_module_init(&self) -> Result<(), LifecycleError> {
        let warmed = self.redis.get("popular_items").await
            .map_err(|e| LifecycleError::new(e.to_string()))?;
        // Store in-memory
        Ok(())
    }
}
```

## When it runs

```
OnModuleConfigure ──► AsyncModuleInit ──► OnModuleInit ──► OnApplicationBootstrap
```

Hooks run in **dependency order** — if module A depends on module B, B's `OnModuleInit` runs first.

## Registration

```rust
ModuleDefinition::builder::<CacheService>()
    .module_init()
    .build()
```

## Error behavior

If `OnModuleInit` returns an error:
- The application startup **fails** with the error message
- All previously initialized modules have their `OnModuleDestroy` called in reverse order
- No half-initialized state is left behind

## Best practices

- **Don't panic** — return `Err(LifecycleError::new(...))` instead
- **Don't leak secrets** — error messages should be safe diagnostics
- **Keep it quick** — long init delays startup; use background tasks if needed
- **Idempotent** — design `OnModuleInit` to be safe if called again
