---
title: OnModuleConfigure
description: Runs during module graph compilation — validate config, register routes dynamically, set up conditional providers.
---

# OnModuleConfigure

Runs during module graph compilation, **before any providers are built**. This is the earliest lifecycle hook.

## When it fires

```
compile_module_graph(root)
    │
    ├─ For each module (topological order):
    │   └─ OnModuleConfigure  ← YOU ARE HERE
    │
    ▼
initialize_eager_providers()
OnModuleInit()
...
```

At this point, the module graph is assembled but no providers have been constructed yet. You receive the module name as a diagnostic string.

## The trait

```rust
pub trait OnModuleConfigure: Send + Sync + 'static {
    fn on_module_configure(&self, module_name: &str) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnModuleConfigure |
|---|---|
| Validate that required env vars exist before any code runs | Fail-fast before expensive init |
| Register routes conditionally based on config | Module graph is assembled, build hasn't started |
| Check that two modules don't conflict | Cross-module validation before providers are built |
| Log module registration for observability | Earliest point to trace what modules are loaded |

## Example — conditional route registration

```rust
#[derive(Injectable)]
pub struct FeatureGate;

impl OnModuleConfigure for FeatureGate {
    fn on_module_configure(&self, module_name: &str) -> LifecycleFuture<'_> {
        let module = module_name.to_owned();
        Box::pin(async move {
            if std::env::var("ENABLE_EXPERIMENTAL").is_err() {
                tracing::warn!("{}: experimental features disabled", module);
            }
            Ok(())
        })
    }
}
```

## What you CAN'T do here

- Access other providers (they aren't built yet)
- Make database queries (no connection pools exist)
- Modify request handling (server isn't configured)

This hook is for **validation and registration decisions only**.
