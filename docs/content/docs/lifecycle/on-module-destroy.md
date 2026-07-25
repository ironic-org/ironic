---
title: OnModuleDestroy
description: Per-module cleanup — close connections, release resources, reverse initialization order
---

# OnModuleDestroy

`OnModuleDestroy` fires during **shutdown** for each module, in **reverse
initialization order** (last initialized = first destroyed).

## Position

```
BeforeShutdown
  │
  ▼
OnModuleDestroy ◀── YOU ARE HERE (reverse init order)
  │
  ▼
OnApplicationShutdown
```

## Trait

```rust
pub trait OnModuleDestroy {
    async fn on_module_destroy(&self) -> Result<(), LifecycleError>;
}
```

## Basic Usage

```rust
#[derive(Injectable)]
pub struct DatabasePool {
    pool: PgPool,
}

impl OnModuleDestroy for DatabasePool {
    async fn on_module_destroy(&self) -> Result<(), LifecycleError> {
        self.pool.close().await;
        tracing::info!("database pool closed");
        Ok(())
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_destroy = [DatabasePool],
)]
pub struct AppModule;
```

## Error Handling

Errors in `OnModuleDestroy` are **logged but do not prevent** other
destroy hooks from running:

```rust
impl OnModuleDestroy for DatabasePool {
    async fn on_module_destroy(&self) -> Result<(), LifecycleError> {
        if let Err(e) = self.pool.close().await {
            // Error is logged, but OTHER destroy hooks STILL RUN
            return Err(LifecycleError::new(format!("close failed: {e}")));
        }
        Ok(())
    }
}
```
