---
title: OnModuleLoad
description: Runtime hook — fires when a module is dynamically loaded after bootstrap for provider initialization.
---

# OnModuleLoad

Runs when a module is **dynamically loaded** at runtime — after the application has already bootstrapped and is serving requests.

## When it fires

```
Application running
    │
    ├─ Container::with_override() adds new provider
    │
    ▼
OnModuleLoad  ← YOU ARE HERE
```

This is the runtime counterpart of `OnModuleInit`. Use it for providers registered after bootstrap via `Container::with_override()`.

## The trait

```rust
pub trait OnModuleLoad: Send + Sync + 'static {
    fn on_module_load(&self, module_name: &str) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnModuleLoad |
|---|---|
| Initialize a provider hot-swapped at runtime | A/B testing different implementations |
| Register dynamic routes for a feature-flag-gated module | Feature turned on without restart |
| Warm caches for a newly loaded service | Provider just added to the container |

## Example — Hot-swapped cache backend

```rust
#[derive(Injectable)]
pub struct RedisCacheLoader;

impl OnModuleLoad for RedisCacheLoader {
    fn on_module_load(&self, module_name: &str) -> LifecycleFuture<'_> {
        let name = module_name.to_owned();
        Box::pin(async move {
            tracing::info!("Loading module: {name}");
            // Initialize Redis connections, register routes, etc.
            Ok(())
        })
    }
}
```

## Registration

```rust
LifecycleDefinition::builder::<RedisCacheLoader>()
    .module_load()
    .build()
```

## Pair with OnModuleUnload

Always implement `OnModuleUnload` when you implement `OnModuleLoad` — it's the cleanup counterpart.
