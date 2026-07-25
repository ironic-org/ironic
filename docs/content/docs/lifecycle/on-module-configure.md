---
title: OnModuleConfigure
description: Dynamic module configuration before providers are built — conditional routes, environment-specific setup
---

# OnModuleConfigure

`OnModuleConfigure` fires during module graph compilation, **before any
providers are constructed**. It's your chance to modify the module definition
based on runtime conditions.

## Position in the Lifecycle

```
COMPILE TIME
  │
  ▼
OnModuleConfigure ◀── YOU ARE HERE
  │
  ▼
AsyncModuleInit
  │
  ▼
OnModuleInit
  │
  ▼
...startup continues...
```

## Trait Signature

```rust
pub trait OnModuleConfigure {
    fn configure(&self, module: &mut ModuleDefinitionBuilder);
}
```

Note: This is a **synchronous** hook. It fires during graph compilation,
before any async runtime is available.

## Basic Usage

```rust
use ironic::prelude::*;

pub struct RouteRegistrar;

impl OnModuleConfigure for RouteRegistrar {
    fn configure(&self, module: &mut ModuleDefinitionBuilder) {
        // Only import the admin module in development
        if cfg!(debug_assertions) {
            module.import::<AdminModule>();
        }
    }
}

// Register via:
#[derive(Module)]
#[module(
    lifecycle_configure = [RouteRegistrar],
)]
pub struct AppModule;
```

## What You Can Do Here

| Operation | Allowed? | Example |
|-----------|----------|---------|
| Add imports | ✅ | `.import::<Module>()` |
| Add providers | ✅ | `.provider(ProviderDefinition::value(...))` |
| Add controllers | ✅ | `.controller(Controller::definition())` |
| Access environment vars | ✅ | `std::env::var("FEATURE_FLAG")` |
| Access config files | ✅ | Read from disk, env, etc. |
| Async operations | ❌ | Hook is synchronous |
| Access DI container | ❌ | Container doesn't exist yet |

## Registration

### Via `#[derive(Module)]`

```rust
#[derive(Module)]
#[module(
    lifecycle_configure = [RouteRegistrar],
)]
pub struct AppModule;
```

### Via `LifecycleDefinition`

```rust
ModuleDefinition::builder::<MyModule>()
    .lifecycle(
        LifecycleDefinition::builder::<RouteRegistrar>()
            .module_configure()
            .build(),
    )
    .build();
```

## Pattern: Feature Flag Routes

```rust
pub struct FeatureFlagRouter;

impl OnModuleConfigure for FeatureFlagRouter {
    fn configure(&self, module: &mut ModuleDefinitionBuilder) {
        // Conditionally import based on environment
        let env = std::env::var("APP_ENV").unwrap_or_default();
        match env.as_str() {
            "production" => {
                module.import::<ProdModule>();
                module.import::<MonitoringModule>();
            }
            "staging" => {
                module.import::<StagingModule>();
            }
            _ => {
                module.import::<DevModule>();
                module.import::<DebugModule>();
            }
        }
    }
}
```

## Pattern: Database-Specific Module Loading

```rust
pub struct DatabaseRouter;

impl OnModuleConfigure for DatabaseRouter {
    fn configure(&self, module: &mut ModuleDefinitionBuilder) {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_default();

        if db_url.contains("postgres") {
            module.import::<PostgresModule>();
        } else if db_url.contains("mysql") {
            module.import::<MysqlModule>();
        } else {
            module.import::<SqliteModule>();
        }
    }
}
```

## How It's Different From Other Hooks

| Hook | Timing | Has Container? | Async? |
|------|--------|---------------|--------|
| `OnModuleConfigure` | Graph compilation | ❌ | ❌ |
| `AsyncModuleInit` | After container built | ✅ | ✅ |
| `OnModuleInit` | After deps resolved | ❌ | ✅ |

Use `OnModuleConfigure` for structural changes. Use `AsyncModuleInit`
or `OnModuleInit` for runtime initialization.
