---
title: OnModuleInit
description: Initialize per-module resources — database connections, cache warmup, seed data. The most commonly used lifecycle hook.
---

# OnModuleInit

## What is it?

`OnModuleInit` is the hook you'll use most. It runs after Ironic's DI container resolves your provider, but **before** the server starts. This is where you set up per-module resources.

**Analogy:** Each department in a store opens independently. The bakery turns on ovens. The electronics department powers up displays. Each department initializes its OWN equipment. That's `OnModuleInit`.

## When it fires

```
compile_module_graph(root)
    │
    ▼
container.resolve(MyService)   ← DI resolves your provider
    │
    ▼
OnModuleInit                   ← YOU ARE HERE
    │
    ▼
[Next module's OnModuleInit]   ← runs AFTER yours
    │
    ...
    ▼
OnApplicationBootstrap          ← ALL modules done
Server starts
```

## Why you need it

Without `OnModuleInit`, you'd have to manually call setup code in `main()`:

```rust
// ❌ Manual — fragile, order-dependent, error-prone
let db = Database::connect(&config.db_url).await;
let cache = Cache::warm(&db).await;
let mailer = Mailer::new(&config.smtp).await;
```

With `OnModuleInit`, each module initializes itself:

```rust
// ✅ Automatic — each module owns its initialization
#[module(
    providers = [Database, Cache, Mailer],
    lifecycle_init = [Database, Cache, Mailer],
)]
struct AppModule;
```

## How to use

```rust
use ironic::{Injectable, LifecycleFuture, OnModuleInit};
use std::sync::Arc;

#[derive(Injectable)]
pub struct Database {
    url: String,
    pool: std::sync::Mutex<Option<ConnectionPool>>,
}

impl OnModuleInit for Database {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            tracing::info!("Connecting to database...");
            let pool = ConnectionPool::connect(&self.url)
                .await
                .map_err(|e| LifecycleError::new(format!("DB connection failed: {e}")))?;
            *self.pool.lock().unwrap() = Some(pool);
            tracing::info!("Database connected");
            Ok(())
        })
    }
}
```

Register it:

```rust
#[derive(Module)]
#[module(
    providers = [Database],
    lifecycle_init = [Database],  // ← register for OnModuleInit
)]
pub struct DatabaseModule;
```

## Execution order

Modules initialize in **topological order**. If `UsersModule` imports `DatabaseModule`, then `DatabaseModule::OnModuleInit` runs BEFORE `UsersModule::OnModuleInit`.

```
DatabaseModule imports nothing → runs FIRST
    │
UsersModule imports DatabaseModule → runs SECOND (depends on DB being ready)
    │
AppModule imports UsersModule → runs LAST
```

This means your `OnModuleInit` can safely access providers from imported modules — they're already initialized.

## Common patterns

### Seed data on first run

```rust
impl OnModuleInit for BlogService {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            if self.count_posts()? == 0 {
                self.create_post("Getting Started", "Welcome!")?;
                tracing::info!("Seeded initial data");
            }
            Ok(())
        })
    }
}
```

### Cache warmup

```rust
impl OnModuleInit for ProductCache {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let products = self.repo.list_all()?;
            for p in products {
                self.cache.set(&p.id, &p).await?;
            }
            tracing::info!("Warmed {} products", products.len());
            Ok(())
        })
    }
}
```

### Run database migrations

```rust
impl OnModuleInit for Migrator {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            sqlx::migrate!("./migrations")
                .run(&*self.pool)
                .await
                .map_err(|e| LifecycleError::new(format!("Migration failed: {e}")))?;
            tracing::info!("Migrations complete");
            Ok(())
        })
    }
}
```

## Error handling

If `OnModuleInit` returns `Err(...)`, the application **aborts startup**. The framework then runs `OnModuleDestroy` in reverse for all providers that successfully initialized:

```
Module A: OnModuleInit → ✅ OK
Module B: OnModuleInit → ✅ OK
Module C: OnModuleInit → ❌ FAIL
──────────────────────────────
Module B: OnModuleDestroy → cleanup
Module A: OnModuleDestroy → cleanup
PROCESS EXITS
```

No resources are leaked — every module that initialized gets its destroy hook called.

## OnModuleInit vs OnApplicationBootstrap

| | OnModuleInit | OnApplicationBootstrap |
|---|---|---|
| **Scope** | Your module only | Entire application |
| **Dependencies** | Imported modules are ready | Every module is ready |
| **Best for** | DB connections, resource setup | Cron jobs, cross-module checks |
| **Order** | Topological (leaves first) | Registration order |
| **On failure** | Reverse destroy for YOUR module | Reverse destroy for ALL modules |

Use `OnModuleInit` when you're setting up YOUR module's resources. Use `OnApplicationBootstrap` when you need OTHER modules to be done first.
