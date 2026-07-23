---
title: OnModuleConfigure
description: Dynamic module setup before any providers are constructed.
---

# OnModuleConfigure

Runs during module graph compilation, **before any providers are built**.

## Use cases

- Dynamic route registration
- Conditional provider setup based on feature flags
- Validating module configuration
- Registering middleware conditionally

## Signature

```rust
pub trait OnModuleConfigure: Send + Sync + 'static {
    fn on_module_configure(&self, module_name: &str) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnModuleConfigure, LifecycleError};

struct RoutingModule;

impl OnModuleConfigure for RoutingModule {
    async fn on_module_configure(&self, _module_name: &str) -> Result<(), LifecycleError> {
        // Register dynamic routes based on feature flags
        Ok(())
    }
}
```

## When it runs

```
Compile time ──► OnModuleConfigure ──► AsyncModuleInit ──► OnModuleInit
```

It's the **first** hook to execute — before any providers exist, before the DI container is fully built.

## Registration

```rust
ModuleDefinition::builder::<RoutingModule>()
    .module_configure()
    .build()
```

## Common patterns

### Conditional route registration

```rust
impl OnModuleConfigure for AdminModule {
    async fn on_module_configure(&self, name: &str) -> Result<(), LifecycleError> {
        if self.config.admin_enabled {
            // Register admin routes dynamically
            register_admin_routes();
        }
        Ok(())
    }
}
```

### Configuration validation

```rust
impl OnModuleConfigure for DatabaseModule {
    async fn on_module_configure(&self, _name: &str) -> Result<(), LifecycleError> {
        if self.config.url.is_empty() {
            return Err(LifecycleError::new("database.url is required"));
        }
        Ok(())
    }
}
```
