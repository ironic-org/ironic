---
title: Dynamic Modules
description: Create modules at runtime based on configuration — import conditionally, register providers dynamically.
---

# Dynamic Modules

## What you'll learn

- Conditionally import modules based on config
- Register providers at runtime
- Create modules from configuration values

---

## Conditional imports

Only import a module if a condition is met:

```rust
#[derive(Module)]
#[module(
    import_if(feature = "cache", [CacheModule]),  // ← Only if cache feature is on
    imports = [CoreModule],
)]
struct AppModule;
```

Or based on runtime configuration:

```rust
if config.enable_analytics {
    app.import_module(AnalyticsModule::definition());
}
```

## Dynamic providers

Register providers at runtime instead of compile time:

```rust
let db_url = config.database_url.clone();
let provider = ProviderDefinition::constructor(
    Scope::Singleton,
    vec![],
    move |_resolver| {
        let pool = PgPool::connect_lazy(&db_url)?;
        Ok(pool)
    },
);
module_def.add_provider(provider);
```

## When to use dynamic modules

| Scenario | Approach |
|----------|----------|
| Feature flag gating | `import_if(feature = "...")` |
| Config-driven features | `import_if(condition = ...)` or runtime checks |
| Database-dependent setup | Dynamic providers with config values |

## What you learned

- [x] `import_if` conditionally includes modules
- [x] Dynamic providers create services from config
- [x] Runtime module registration with `.import_module()`
