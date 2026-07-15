---
title: Dynamic Modules
description: Create modules at runtime based on configuration — import conditionally, register providers dynamically.
---

# Dynamic Modules

## What you'll learn

- Conditionally import modules based on compile-time features and runtime config
- Register providers dynamically at runtime
- Load async configuration from external sources
- Avoid common dynamic-module anti-patterns
- Test modules with mocked configuration

---

## Conditional imports

Only import a module if a compile-time feature is enabled:

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

## Async module configuration

Load configuration from external sources before registering modules:

```rust
async fn configure_app() -> ironic::Application {
    let secrets = SecretsManager::from_env().await?;
    let redis_url = secrets.get("REDIS_URL")?;

    let mut app = ironic::Application::new();
    app.import_module(CoreModule::definition());

    if let Ok(url) = redis_url {
        let redis_provider = ProviderDefinition::constructor(
            Scope::Singleton,
            vec![],
            move |_resolver| {
                let client = redis::Client::open(&url)?;
                Ok(client)
            },
        );
        app.add_provider(redis_provider);
    }

    app
}
```

Common async config sources:

| Source | Use case |
|--------|----------|
| Environment variables | `std::env::var("KEY")` — simple, single-machine |
| Secrets manager (Vault, AWS) | Production secrets, rotation |
| Service discovery (Consul, etcd) | Multi-service deployments |
| Remote config server | Centralized feature flags |

## Conditional module registration

Use feature flags or runtime conditions to decide which modules to register:

```rust
impl Module for ConditionalApp {
    fn definition(&self) -> ModuleDefinition {
        let mut def = ModuleDefinition::new("conditional_app");

        def.import(CoreModule::definition());

        if std::env::var("ENABLE_CACHE").is_ok() {
            def.import(CacheModule::definition());
        }

        if self.config.is_production() {
            def.import(ProdModule::definition());
        } else {
            def.import(DevModule::definition());
        }

        def
    }
}
```

## Anti-patterns

Doing these will cause trouble:

| Anti-pattern | Problem | Instead |
|-------------|---------|---------|
| Rebuilding the container per request | Massive performance overhead | Build once at startup; use scoped providers |
| Circular dynamic imports | Module A imports B, B dynamically imports A | Extract shared deps into a third module |
| Importing modules from inside providers | Unpredictable resolution order | Only import modules during app bootstrap |
| Heavy I/O in provider constructors | Blocks startup, no retry mechanism | Use lazy initialization inside the service |
| Ignoring `import_if` edge cases | Runtime-only checks don't show compile errors for missing types | Pair `import_if` with `#[cfg(feature = "...")]` for type-safe code |

## Scope interaction

Dynamic modules participate in Ironic's scope system normally. The key rule is:

```rust
// This works: dynamic singleton depends on static singleton
let dynamic_provider = ProviderDefinition::constructor(
    Scope::Singleton,
    vec![PgPool::dependency_token()],
    move |resolver| {
        let pool: &PgPool = resolver.resolve();
        Ok(Cache::new(pool))
    },
);

// This FAILS at runtime: dynamic singleton cannot depend on request-scoped
let bad_provider = ProviderDefinition::constructor(
    Scope::Singleton,
    vec![UserContext::dependency_token()], // ← Compiler catch, not runtime
    move |resolver| { /* ... */ },
);
```

The same scope rules apply to dynamic modules as static ones — singletons can't depend on request-scoped providers, and the DI container enforces it.

## Lazy loading vs eager loading

| Strategy | How it works | Tradeoff |
|----------|-------------|----------|
| **Eager** (default) | All modules loaded at startup | Fast first request, but slower boot |
| **Lazy** | Module loaded on first use | Fast boot, but slow first request |

For dynamic modules, lazy loading is often preferred since you may not know at compile time whether a module is needed:

```rust
app.import_module_lazy(SearchModule::definition());
// SearchModule and its providers are only instantiated
// when SearchService is first injected.
```

## Testing dynamic modules

Mock the configuration source to test dynamic module behavior:

```rust
#[test]
fn analytics_module_not_loaded_when_disabled() {
    let mut config = AppConfig::default();
    config.enable_analytics = false;

    let app = build_app_from_config(config);
    let modules = app.list_modules();

    assert!(!modules.contains(&"analytics"));
}

#[test]
fn analytics_module_loaded_when_enabled() {
    let mut config = AppConfig::default();
    config.enable_analytics = true;

    let app = build_app_from_config(config);
    let providers = app.resolve_all::<AnalyticsService>();

    assert!(providers.is_ok());
}
```

Wrap your config loading behind a trait so you can swap a mock in tests:

```rust
trait ConfigSource {
    fn get_database_url(&self) -> Result<String>;
}

fn build_app(config: &dyn ConfigSource) -> ironic::Application {
    // Use config.get_database_url() in provider constructors
}
```

## Common mistakes

| Mistake | Why it hurts | Fix |
|---------|-------------|-----|
| Not testing `import_if` branches | Config-driven code paths never exercised | Write a test per branch |
| Loading secrets at compile time | Secrets baked into the binary | Use dynamic providers that read env vars at startup |
| Forgetting to add dynamic module deps | Missing provider errors at runtime | Explicitly list deps in `ProviderDefinition` |
| Using dynamic imports for everything | Slower startup, harder to reason about | Use static imports for always-on modules |
| Mixing `import_if` and runtime conditionals for the same module | Race condition: compile check passes, runtime import fails | Pick one strategy per module |

## When to use dynamic modules

| Scenario | Approach |
|----------|----------|
| Feature flag gating | `import_if(feature = "...")` |
| Config-driven features | `import_if(condition = ...)` or runtime checks |
| Database-dependent setup | Dynamic providers with config values |
| Async secrets loading | `async` bootstrap + dynamic provider registration |
| Test-only module swaps | Trait-based config source + mock implementations |

## What you learned

- [x] `import_if` conditionally includes modules at compile time
- [x] Dynamic providers create services from config at runtime
- [x] Runtime module registration with `.import_module()`
- [x] Async bootstrap loads secrets and service discovery config
- [x] Dynamic modules follow the same scope rules as static modules
- [x] Test dynamic modules by mocking the config source
- [x] Avoid anti-patterns: no circular imports, no per-request rebuilds
