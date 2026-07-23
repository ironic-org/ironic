---
title: OnModuleDestroy
description: Per-module cleanup during shutdown — reverse initialization order.
---

# OnModuleDestroy

Runs during shutdown for each successfully initialized module. Modules are destroyed in **reverse** initialization order.

## Use cases

- Closing database connections
- Flushing buffers and queues
- Stopping background workers
- Releasing file handles and network resources
- Writing final state to disk

## Signature

```rust
pub trait OnModuleDestroy: Send + Sync + 'static {
    fn on_module_destroy(&self) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnModuleDestroy, LifecycleError};

struct DatabaseService {
    pool: PgPool,
}

impl OnModuleDestroy for DatabaseService {
    async fn on_module_destroy(&self) -> Result<(), LifecycleError> {
        self.pool.close().await;
        tracing::info!("Database pool closed");
        Ok(())
    }
}
```

## When it runs

```
BeforeShutdown  -->  OnModuleDestroy  -->  OnApplicationShutdown  -->  AfterShutdown
```

Modules are destroyed in **reverse initialization order** — the last module initialized is the first destroyed.

## Registration

```rust
ModuleDefinition::builder::<DatabaseService>()
    .module_destroy()
    .build()
```

## Error behavior

Errors in `OnModuleDestroy` are **logged but don't prevent** other modules from being destroyed. The shutdown continues through all remaining modules.

## Best practices

- Always make `OnModuleDestroy` **idempotent** — it may be called during startup rollback too
- Use timeouts for destroy operations that could hang
- Log errors but don't panic — a failed destroy shouldn't prevent cleanup of other modules
- Pair every `OnModuleInit` with a corresponding `OnModuleDestroy`
