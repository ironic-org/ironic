---
title: DevTools & Plugins
description: Explore your app visually — module graph, route inspector, and plugin system for extensibility.
---

# DevTools & Plugins

## What you'll learn

- Enable the DevTools dashboard
- Inspect modules, routes, and providers visually
- Create reusable plugins with the `Plugin` trait
- Understand plugin lifecycle (load, register, shutdown)
- Configure plugins via `ironic.toml`

Enable in `Cargo.toml`:

```toml
ironic = { features = ["devtools"] }
```

---

## DevTools UI

Visit `http://localhost:3000/dev` to see:

- **Module graph** — visual map of all modules and their dependencies
- **Route inspector** — every route with its HTTP method, path, and handler
- **Provider list** — all registered services with their scope and dependencies

## Plugin lifecycle

Plugins go through four stages from load to shutdown:

| Stage | Hook | Description |
|-------|------|-------------|
| 1. Discovery | (automatic) | Ironic scans declared plugin types at startup |
| 2. Configuration | `configure()` | Plugin reads its config section from `ironic.toml` |
| 3. Registration | `register()` | Plugin adds modules, middleware, and routes to the app |
| 4. Shutdown | `shutdown()` | Plugin releases resources (connections, file handles) |

## Plugin trait in detail

```rust
use ironic::Plugin;

pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;

    fn register(&self, registry: &mut ironic::PluginRegistry);

    fn configure(&self, config: &ironic::PluginConfig) -> Result<(), ironic::PluginError> {
        Ok(())  // Optional — override if your plugin reads config
    }

    fn shutdown(&self) {
        // Optional — override for cleanup
    }
}
```

Required methods:
- `name()` — unique identifier, used in logs and `ironic.toml`
- `register()` — where you attach modules, middleware, and routes

Optional methods:
- `configure()` — called before registration; read plugin-specific config here
- `shutdown()` — called on graceful shutdown; close connections, flush buffers

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

## Plugin configuration via ironic.toml

Plugins can read their own section from the app config:

```toml
# ironic.toml
[plugins.audit]
storage = "postgres"
retention_days = 90
log_request_body = true
```

```rust
impl Plugin for AuditPlugin {
    fn configure(&self, config: &ironic::PluginConfig) -> Result<(), ironic::PluginError> {
        let storage = config.get_str("storage")?;
        let retention = config.get_int("retention_days")? as u32;
        Ok(())
    }
}
```

Config keys are namespaced under `[plugins.<your_plugin_name>]` — no risk of collision with other plugins.

## Building a real plugin: step by step

Here's a complete rate-limiter plugin that counts requests per client:

```rust
use ironic::Plugin;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct RateLimiterPlugin {
    counters: Arc<Mutex<HashMap<String, u64>>>,
}

impl RateLimiterPlugin {
    pub fn new() -> Self {
        Self { counters: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl Plugin for RateLimiterPlugin {
    fn name(&self) -> &str { "rate_limiter" }

    fn register(&self, registry: &mut ironic::PluginRegistry) {
        let counters = self.counters.clone();
        registry.add_middleware(move |req, next| {
            let ip = req.client_ip();
            let mut map = counters.lock().unwrap();
            let count = map.entry(ip).and_modify(|c| *c += 1).or_insert(1);

            if *count > 100 {
                ironic::Response::too_many_requests("Rate limit exceeded")
            } else {
                next.call(req)
            }
        });
    }

    fn shutdown(&self) {
        let mut map = self.counters.lock().unwrap();
        let total: u64 = map.values().sum();
        tracing::info!("Rate limiter shutting down. Total requests tracked: {total}");
        map.clear();
    }
}
```

Steps to create any plugin:

1. Define a struct that holds plugin state
2. Implement `Plugin` — at minimum `name()` and `register()`
3. Override `configure()` if the plugin needs config from `ironic.toml`
4. Override `shutdown()` if the plugin owns resources
5. Register it with `PluginRegistry::new().plugin(YourPlugin)`

## Error handling in plugins

When a plugin fails to load, Ironic logs the error and skips it:

```rust
fn register(&self, registry: &mut ironic::PluginRegistry) {
    // If AuditModule::definition() panics, the plugin is skipped,
    // the app continues without it, and the error is logged.
    registry.add_module(AuditModule::definition());
}
```

For critical plugins that the app cannot run without, make the error fatal yourself:

```rust
fn configure(&self, config: &ironic::PluginConfig) -> Result<(), ironic::PluginError> {
    let db_url = config.get_str("database_url")
        .map_err(|_| ironic::PluginError::fatal("audit", "database_url is required"))?;
    Ok(())
}
```

A `PluginError::fatal` causes the application to refuse to start, with a clear error message showing which plugin failed and why.

## Common mistakes

| Mistake | Why it hurts | Fix |
|---------|-------------|-----|
| Blocking in `register()` | Delays app startup for every plugin | Keep registration logic fast; defer heavy work to first use |
| Forgetting to implement `shutdown()` | Resource leaks — open connections, temp files | Always clean up resources you allocate |
| Panicking in `configure()` | Plugin silently skipped, missing functionality | Return `PluginError` instead of panicking |
| Plugin name collision | Second plugin overwrites first's config | Use unique, namespaced names (e.g., `acme/redis-cache`) |
| No config defaults | Plugin fails to load if `ironic.toml` has no `[plugins.your_plugin]` | Provide sensible defaults in `configure()` |

## What you learned

- [x] DevTools UI shows modules, routes, and providers at `/dev`
- [x] Plugins package functionality for reuse with `Plugin::register()`
- [x] `PluginRegistry` manages plugin lifecycle (load → configure → register → shutdown)
- [x] `ironic.toml` configures plugins under `[plugins.<name>]`
- [x] Return `PluginError::fatal` for required configuration
- [x] Override `shutdown()` to clean up resources
