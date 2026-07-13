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

```rust,ignore
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

```rust,ignore
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
