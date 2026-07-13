---
title: Dynamic modules
description: Compose lazy, conditional, runtime-created, and asynchronously configured modules.
---

# Dynamic modules

Ironic keeps one validated module graph regardless of how definitions are assembled.

Static imports are lazy: `import::<FeatureModule>()` stores the module factory and expands it only
when the application graph is compiled. `import_lazy` is an explicit alias when that behavior is
important to the reader.

## Conditional composition

```text
ModuleDefinition::builder::<AppModule>()
    .import_if::<MetricsModule>(settings.metrics_enabled)
    .provider_if(settings.local_cache, LocalCache::provider_definition())
    .controller_if(settings.admin_api, AdminController::definition())
    .build()
```

Disabled definitions never enter validation, DI registration, route discovery, or lifecycle
execution.

## Runtime-created imports

Build a normal `ModuleDefinition` from runtime configuration and attach it with
`import_definition`. It retains the module type's stable identity, so duplicate imports, cycles,
visibility, and lifecycle ordering receive the same validation as static modules.

## Asynchronous configuration

Use `module_async` when the root graph depends on an asynchronous source:

```text
FrameworkApplication::builder()
    .module_async(async move {
        let settings = settings_service.load().await
            .map_err(|_| ModuleConfigurationError::new("settings are unavailable"))?;
        Ok(build_root_module(settings))
    })
    .platform(AxumAdapter::default())
    .build()
    .await?;
```

Return only safe messages from `ModuleConfigurationError`; configuration failures can be logged or
rendered and must not expose secret values.

## Global modules

Annotate a module with `#[global]` to make its exported providers visible to every module in the
application without explicit imports:

```rust
use ironic::Module;

#[global]
#[module(providers = [DatabaseConnection], exports = [DatabaseConnection])]
struct DatabaseModule;
```

Global providers injected anywhere:

```rust
#[module(providers = [ReportingService])]
struct ReportsModule;

#[injectable]
struct ReportingService {
    connection: Arc<DatabaseConnection>, // visible without importing DatabaseModule
}
```

## ModuleRef — runtime container access

Inject `ModuleRef` to resolve providers dynamically at runtime:

```rust
use ironic::ModuleRef;

#[injectable]
struct LazyService {
    module_ref: Arc<ModuleRef>,
}

impl LazyService {
    async fn resolve_something(&self) -> Result<Arc<EmailSender>, ResolveError> {
        self.module_ref.resolve::<EmailSender>().await
    }
}
```

`ModuleRef::resolve_optional()` returns `None` for unregistered providers instead of failing:

```rust
let maybe_logger = self.module_ref.resolve_optional::<Logger>().await?;
```

## Parameterized modules (forRoot / register)

Use static methods to accept configuration and return a composed `ModuleDefinition`:

```rust
impl DatabaseModule {
    pub fn for_root(config: DatabaseConfig) -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .provider(ProviderDefinition::value(config))
            .export::<DatabaseConfig>()
            .build()
    }
}

// Use it as an import:
ModuleDefinition::builder::<AppModule>()
    .import_definition(DatabaseModule::for_root(DatabaseConfig::new()))
    .build()
```

The same pattern supports `for_root_async`, `register`, or any custom static method that
returns a `ModuleDefinition`. Use `import_definition()` to attach the result to any parent module.
