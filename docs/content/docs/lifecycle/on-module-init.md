---
title: OnModuleInit
description: Per-provider initialization after all dependencies are resolved — migrations, seeding, background tasks
---

# OnModuleInit

`OnModuleInit` fires **after all dependencies are resolved** and the provider
is fully constructed. All `Arc<T>` fields are populated and ready to use.

## Position

```
AsyncModuleInit
  │
  ▼
OnModuleInit ◀── YOU ARE HERE (all deps ready to use)
  │
  ▼
OnApplicationBootstrap
  │
  ▼
OnServerReady
```

## Trait

```rust
pub trait OnModuleInit {
    async fn on_module_init(&self) -> Result<(), LifecycleError>;
}
```

## Basic Usage

```rust
#[derive(Injectable)]
pub struct MigrationRunner {
    pool: Arc<DatabasePool>,  // fully resolved
}

impl OnModuleInit for MigrationRunner {
    async fn on_module_init(&self) -> Result<(), LifecycleError> {
        // Dependencies are safe to use
        self.pool.run_migrations().await
            .map_err(|e| LifecycleError::new(format!("migration failed: {e}")))?;

        tracing::info!("database migrations complete");
        Ok(())
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_init = [MigrationRunner],
)]
pub struct AppModule;
```

## Common Uses

| Use Case | Example |
|----------|---------|
| Run DB migrations | `pool.run_migrations().await` |
| Seed initial data | `repository.seed_defaults().await` |
| Start background workers | `tokio::spawn(worker.run())` |
| Warm caches | `cache.preload_popular_items().await` |
| Validate external services | `health_check.ping_external_apis().await` |
| Register metrics | `metrics.register_counter("requests_total")` |
