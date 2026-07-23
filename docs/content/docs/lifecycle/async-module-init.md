---
title: AsyncModuleInit
description: Container-aware async initialization — for database connections and migrations.
---

# AsyncModuleInit

Runs after the DI container is built but before any per-provider lifecycle hooks fire. Unlike `OnModuleInit`, this trait **receives the full container** so it can resolve providers during initialization.

## Use cases

- Connecting to databases and running migrations
- Initializing services that depend on multiple providers
- Setting up infrastructure that needs the container to resolve dependencies
- Registering eager singletons at startup

## Signature

```rust
pub trait AsyncModuleInit: Send + Sync + 'static {
    fn async_init<'a>(&'a self, container: &'a Container) -> LifecycleFuture<'a>;
}
```

## Example

```rust
use ironic::{AsyncModuleInit, LifecycleError, Container};
use sqlx::PgPool;

struct DatabaseModule;

impl AsyncModuleInit for DatabaseModule {
    async fn async_init(&self, container: &Container) -> Result<(), LifecycleError> {
        let url = container.resolve::<DatabaseConfig>().await
            .map_err(|e| LifecycleError::new(e.to_string()))?;

        let pool = PgPool::connect(&url.url).await
            .map_err(|e| LifecycleError::new(e.to_string()))?;

        sqlx::migrate!().run(&pool).await
            .map_err(|e| LifecycleError::new(format!("Migration failed: {e}")))?;

        container.register_eager(pool).await;
        Ok(())
    }
}
```

## When it runs

```
OnModuleConfigure ──► AsyncModuleInit ──► OnModuleInit ──► OnApplicationBootstrap
```

It runs once per module, **before** any `OnModuleInit` hooks.

## Key difference from OnModuleInit

| Aspect | OnModuleInit | AsyncModuleInit |
|--------|-------------|-----------------|
| Container access | No | Yes |
| Runs per provider | Yes | Once per module |
| Use case | Simple initialization | Complex setup requiring DI |

## Registration

```rust
ModuleDefinition::builder::<DatabaseModule>()
    .async_init()
    .build()
```

## Best practices

- Use `AsyncModuleInit` for **one-time** module-level setup
- Use `OnModuleInit` for per-provider initialization
- Don't register providers from within `async_init` — use the container's `register_eager` instead
