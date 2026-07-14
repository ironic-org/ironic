---
title: DevTools & Plugins
description: Explore your app visually — module graph, route inspector, and plugin system for extensibility.
---

# DevTools & Plugins

Enable in `Cargo.toml`:

```toml
ironic = { features = ["devtools"] }
```

## DevTools UI

Visit `http://localhost:3000/dev` to see:

- **Module graph** — visual map of all modules and their dependencies
- **Route inspector** — every route with its HTTP method, path, and handler
- **Provider list** — all registered services with their scope and dependencies

## Plugin system

Create reusable plugins that add functionality to any app:

```rust
use ironic::Plugin;

struct AuditPlugin;

impl Plugin for AuditPlugin {
    fn name(&self) -> &str { "audit" }

    fn register(&self, app: &mut ironic::PluginRegistry) {
        app.add_module(AuditModule::definition());
        app.add_middleware(AuditMiddleware);
    }
}

// In your main:
let registry = PluginRegistry::new()
    .plugin(AuditPlugin)
    .plugin(MetricsPlugin);
```

## What you learned

- [x] DevTools UI shows modules, routes, and providers
- [x] Plugins package functionality for reuse
- [x] `PluginRegistry` manages plugin lifecycle
