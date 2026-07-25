---
title: OnModuleLoad / OnModuleUnload
description: Dynamic module lifecycle — initialize and clean up lazy-loaded modules
---

# OnModuleLoad / OnModuleUnload

These hooks fire when a **lazy module** is loaded or unloaded at runtime
via `ModuleRef::load::<T>()`.

## Position

```
Runtime
  │
  ├── ModuleRef::load::<T>() ──▶ OnModuleLoad ◀── fires for loaded module
  │
  └── Module unloaded ─────────▶ OnModuleUnload ◀── fires for unloaded module
```

## Traits

```rust
pub trait OnModuleLoad {
    async fn on_module_load(&self, module_id: &ModuleId);
}

pub trait OnModuleUnload {
    async fn on_module_unload(&self, module_id: &ModuleId);
}
```

## Basic Usage

```rust
pub struct DynamicModuleTracker;

impl OnModuleLoad for DynamicModuleTracker {
    async fn on_module_load(&self, module_id: &ModuleId) {
        tracing::info!("lazy module loaded: {}", module_id.type_name());
        metrics::counter!("modules_loaded_total", 1);
    }
}

impl OnModuleUnload for DynamicModuleTracker {
    async fn on_module_unload(&self, module_id: &ModuleId) {
        tracing::info!("lazy module unloaded: {}", module_id.type_name());
        metrics::counter!("modules_unloaded_total", 1);
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_module_load = [DynamicModuleTracker],
    lifecycle_module_unload = [DynamicModuleTracker],
)]
pub struct AppModule;
```

## Triggering

```rust
// Lazy module is loaded — OnModuleLoad fires
module_ref.load::<AnalyticsModule>().await?;
```
