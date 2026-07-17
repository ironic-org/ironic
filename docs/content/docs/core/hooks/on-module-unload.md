---
title: OnModuleUnload
description: Runtime hook — fires when a module is dynamically unloaded for cleanup before provider removal.
---

# OnModuleUnload

Runs when a module is **dynamically unloaded** at runtime. The cleanup counterpart of `OnModuleLoad`.

## When it fires

```
Runtime
    │
    ├─ Container unregisters provider
    │
    ▼
OnModuleUnload  ← YOU ARE HERE
    │
    ▼
Provider removed from DI container
```

## The trait

```rust
pub trait OnModuleUnload: Send + Sync + 'static {
    fn on_module_unload(&self, module_name: &str) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnModuleUnload |
|---|---|
| Close connections for a hot-swapped provider | Old implementation being replaced |
| Deregister dynamic routes | Feature flag turned off |
| Flush metrics for a removed module | Data cleanup before the provider is dropped |

## OnModuleLoad vs OnModuleUnload

| | OnModuleLoad | OnModuleUnload |
|---|---|---|
| **When** | After provider added to container | Before provider removed |
| **Purpose** | Initialize resources | Clean up resources |
| **Error handling** | Failure aborts the load | Best-effort cleanup |

## Registration

```rust
LifecycleDefinition::builder::<RedisCacheLoader>()
    .module_load()
    .module_unload()
    .build()
```

Always pair `module_load()` with `module_unload()` — they're designed to work together for runtime module lifecycle.
