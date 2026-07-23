---
title: OnApplicationBootstrap
description: Application-wide initialization after all modules are ready.
---

# OnApplicationBootstrap

Runs after **every module's** initialization hooks have succeeded. This is the last step before the server starts listening.

## Use cases

- Seeding default data into the database
- Warming up caches with data from multiple services
- Performing final validation of the application graph
- Starting background task processors
- Registering periodic timers

## Signature

```rust
pub trait OnApplicationBootstrap: Send + Sync + 'static {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnApplicationBootstrap, LifecycleError};

struct AppModule {
    db: DatabaseService,
    cache: CacheService,
}

impl OnApplicationBootstrap for AppModule {
    async fn on_application_bootstrap(&self) -> Result<(), LifecycleError> {
        // Seed default roles if they don't exist
        self.db.seed_roles().await
            .map_err(|e| LifecycleError::new(e.to_string()))?;

        // Warm the cache with data from the database
        let popular = self.db.get_popular_items().await
            .map_err(|e| LifecycleError::new(e.to_string()))?;
        self.cache.warm(popular).await;

        Ok(())
    }
}
```

## When it runs

```
OnModuleInit ──► OnApplicationBootstrap ──► [ Server starts ] ──► OnServerReady
```

## Registration

```rust
ModuleDefinition::builder::<AppModule>()
    .application_bootstrap()
    .build()
```

## Error behavior

If `OnApplicationBootstrap` returns an error, the application **fails to start** and performs graceful shutdown of all initialized modules.

## Best practices

- Use `OnApplicationBootstrap` for **cross-module** initialization
- Keep it **fast** — the server won't start until this completes
- If you have long-running initialization (e.g., warming a large cache), consider doing it in a background task
