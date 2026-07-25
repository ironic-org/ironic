---
title: AsyncModuleInit
description: Container-aware asynchronous initialization — database connections, external service setup
---

# AsyncModuleInit

`AsyncModuleInit` fires **after the container is built** but **before
singletons are resolved**. It receives a reference to the container,
allowing you to resolve other providers during initialization.

## Position in the Lifecycle

```
OnModuleConfigure
  │
  ▼
AsyncModuleInit ◀── YOU ARE HERE (container exists, singletons not yet resolved)
  │
  ▼
OnModuleInit (dependencies resolved)
  │
  ▼
...startup continues...
```

## Trait Signature

```rust
pub trait AsyncModuleInit {
    async fn async_init(&self, container: &Container) -> Result<(), LifecycleError>;
}
```

## Basic Usage

```rust
use ironic::prelude::*;

#[derive(Injectable)]
pub struct DatabaseService;

impl AsyncModuleInit for DatabaseService {
    async fn async_init(&self, container: &Container) -> Result<(), LifecycleError> {
        // Resolve config from the container
        let config = container
            .resolve::<DatabaseConfig>()
            .await
            .map_err(|e| LifecycleError::new(format!("config missing: {e}")))?;

        tracing::info!("connecting to database at {}", config.url);
        // ... connect and store the pool
        Ok(())
    }
}

// Register:
#[derive(Module)]
#[module(
    async_init = [DatabaseService],
)]
pub struct AppModule;
```

## Key Difference from `OnModuleInit`

| Aspect | `AsyncModuleInit` | `OnModuleInit` |
|--------|-------------------|----------------|
| Has `&Container` | ✅ | ❌ |
| Can resolve providers | ✅ | ❌ |
| Runs before singletons | ✅ | ❌ (after) |
| Use case | Set up state that other services need | Use fully-constructed services |

## What You Can Do

```rust
impl AsyncModuleInit for MyService {
    async fn async_init(&self, container: &Container) -> Result<(), LifecycleError> {
        // ✅ Resolve configuration
        let config: Arc<AppConfig> = container.resolve().await?;

        // ✅ Create external connections
        let client = ExternalService::connect(&config.url).await?;

        // ✅ Store for later use (e.g., inject into another service)
        // (store in a shared state that other providers can access)

        // ❌ Cannot use injected dependencies (they don't exist yet)
        // self.other_service.do_something().await;  // WOULD PANIC

        Ok(())
    }
}
```

## Pattern: Connection Pool Setup

```rust
#[derive(Injectable)]
pub struct DatabasePool {
    pool: Option<PgPool>,  // None until initialized
}

impl AsyncModuleInit for DatabasePool {
    async fn async_init(&self, container: &Container) -> Result<(), LifecycleError> {
        let config: Arc<DatabaseConfig> = container.resolve().await
            .map_err(|e| LifecycleError::new(format!("config: {e}")))?;

        let pool = PgPool::connect(&config.url).await
            .map_err(|e| LifecycleError::new(format!("connect: {e}")))?;

        // Store pool via interior mutability or return it
        // (Arc<Mutex<Option<PgPool>>> pattern)
        Ok(())
    }
}
```

## Error Handling

If `async_init` returns an error, the application startup fails:

```
Application::build() will return Err(...) with the lifecycle error.
All previously initialized modules run their destroy hooks in reverse.
```
