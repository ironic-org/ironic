---
title: OnModuleLoad & OnModuleUnload
description: Dynamic module lifecycle hooks for hot-swappable modules.
---

# OnModuleLoad & OnModuleUnload

These hooks fire when modules are dynamically loaded or unloaded at runtime, **after** the application has already started.

## Use cases

- Plugin systems that load modules at runtime
- Feature gate toggling without restart
- Hot-reload of service implementations
- Dynamic route registration/deregistration

## Signatures

```rust
pub trait OnModuleLoad: Send + Sync + 'static {
    fn on_module_load(&self, module_name: &str) -> LifecycleFuture<'_>;
}

pub trait OnModuleUnload: Send + Sync + 'static {
    fn on_module_unload(&self, module_name: &str) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnModuleLoad, OnModuleUnload, LifecycleError};

struct PluginManager;

impl OnModuleLoad for PluginManager {
    async fn on_module_load(&self, module_name: &str) -> Result<(), LifecycleError> {
        tracing::info!("Module loaded: {}", module_name);
        metrics::counter!("modules.loaded").increment(1);
        Ok(())
    }
}

impl OnModuleUnload for PluginManager {
    async fn on_module_unload(&self, module_name: &str) -> Result<(), LifecycleError> {
        tracing::info!("Module unloaded: {}", module_name);
        metrics::counter!("modules.unloaded").increment(1);
        Ok(())
    }
}
```

## Dynamic module lifecycle

```
[Application running]
    |
    +-- Module loaded at runtime --> OnModuleLoad
    |
    +-- Module unloaded at runtime --> OnModuleUnload
```

## Registration

```rust
ModuleDefinition::builder::<PluginManager>()
    .module_load()
    .module_unload()
    .build()
```

## Plugin system pattern

```rust
struct PluginHost {
    loaded_plugins: Vec<Box<dyn Plugin>>,
}

impl OnModuleLoad for PluginHost {
    async fn on_module_load(&self, name: &str) -> Result<(), LifecycleError> {
        if let Some(plugin) = self.hot_reload_plugin(name).await {
            plugin.on_activate().await
                .map_err(|e| LifecycleError::new(e.to_string()))?;
        }
        Ok(())
    }
}
```

## Best practices

- Use `OnModuleLoad` for setup that runs when a module is hot-loaded
- Use `OnModuleUnload` for cleanup that must happen when a module is removed
- These are distinct from startup/shutdown hooks — they run mid-lifecycle
- Not all modules support dynamic loading/unloading (check your module system)
