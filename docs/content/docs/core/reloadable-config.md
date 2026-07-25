---
title: Hot-Reload Config
description: Inject runtime-updating configuration values into providers with Reloadable<T> — no restarts needed.
---

# Hot-Reload Config Injection

`Reloadable<T>` wraps a `tokio::sync::watch::Receiver<T>` so providers always
read the latest configuration value — **without restarting the application**.

## The Problem

Traditional configuration is loaded once at startup:

```rust
let config = load_config("config.json").await;
let pool = DatabasePool::new(&config.database.url);
// ... if config changes, you must restart to pick it up
```

With `Reloadable<T>`, your services always see the latest value:

```rust
#[derive(Injectable)]
struct DatabaseService {
    config: Reloadable<AppConfig>,
}

impl DatabaseService {
    async fn get_pool_size(&self) -> u32 {
        // Always reads the current value — no restart needed
        self.config.latest().database.pool_size
    }
}
```

## How It Works

```
Config File        ConfigurationLoader         watch::Channel           Services
                   
config.json ──▶  ┌──────────────────┐       ┌──────────────┐       ┌────────────┐
                 │  load::<AppConfig>│ ──▶   │     tx       │       │ Service A  │
file watcher     │                  │       │  ──────────▶ │       │ .latest()  │
   │             │  deserialize     │       │     rx       │       │            │
   │             └──────────────────┘       └──────┬───────┘       │ Service B  │
   ▼                                              │               │ .latest()  │
file changes ──▶ reload ──▶ tx.send(new)          │               └────────────┘
                                                  ▼
                                           All .latest() calls
                                           return the new value
```

## Setup

### 1. Create a `ConfigurationLoader` with file watching

```rust
use ironic::config::ConfigurationLoader;
use std::time::Duration;

let loader = ConfigurationLoader::new()
    .file("config.json")
    .watch()  // enables file watching for hot-reload
    .watch_interval(Duration::from_secs(2));
```

### 2. Load the initial config and create a watch channel

```rust
let config = loader.load::<AppConfig>()?;

// Create a watch channel with the initial value
let (tx, rx) = tokio::sync::watch::channel(config);

// Wrap the receiver in Reloadable
let reloadable = Reloadable::new(rx);
```

### 3. Register `Reloadable<T>` as a DI provider

```rust
use ironic::ProviderDefinition;

Application::builder()
    .module(AppModule)
    .provider(ProviderDefinition::value(reloadable))
    .platform(AxumAdapter::new())
    .build()
    .await?;
```

### 4. Inject `Reloadable<T>` in any service

```rust
#[derive(Injectable)]
struct RateLimiter {
    config: Reloadable<AppConfig>,
}

impl RateLimiter {
    fn max_requests(&self) -> u64 {
        self.config.latest().rate_limit.max_requests
    }
}
```

## File Watching Loop

Start a background task that reloads the config on changes:

```rust
use ironic::config::ConfigurationLoader;
use tokio::sync::watch;

async fn watch_config(
    loader: ConfigurationLoader,
    tx: watch::Sender<AppConfig>,
) {
    loop {
        // Wait for file changes
        if loader.changed().await.is_ok() {
            // Reload and send new config
            if let Ok(new_config) = loader.load::<AppConfig>() {
                let _ = tx.send(new_config);
                tracing::info!("configuration reloaded");
            }
        }
    }
}

// Start the watcher in the background
tokio::spawn(watch_config(loader, tx));
```

## With `Container::with_override()`

Combine hot-reload with container overrides for full runtime reconfiguration:

```rust
use ironic::{Container, ProviderDefinition};

async fn reconfigure_on_change(
    mut container: Container,
    mut rx: watch::Receiver<AppConfig>,
) {
    while rx.changed().await.is_ok() {
        let new_config = rx.borrow().clone();

        // Override configuration provider
        container = container.with_override(
            ProviderDefinition::value(new_config)
        );

        tracing::info!("container reconfigured with new settings");
    }
}
```

## API Reference

| Method | Returns | Description |
|--------|---------|-------------|
| `latest()` | `T` | Most recent config value (cheap clone via `Arc`) |
| `receiver()` | `&watch::Receiver<T>` | Raw receiver for custom watch logic |
| `changed().await` | `Result<(), RecvError>` | Waits for next update |

```rust
impl Reloadable<AppConfig> {
    // Get current value
    let current = self.config.latest();

    // Or use the raw receiver for complex scenarios
    let mut rx = self.config.receiver().clone();
    tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            println!("config changed: {:?}", rx.borrow());
        }
    });
}
```

## When to Use

| Scenario | Example | Reloadable? |
|----------|---------|-------------|
| Database pool size | `latest().database.pool_size` | ✅ |
| Feature flags | `latest().features.enable_new_pipeline` | ✅ |
| Rate limits | `latest().rate_limit.max_requests` | ✅ |
| Log levels | `latest().logging.level` | ✅ |
| Database URL | `latest().database.url` | ❌ needs reconnection |
| TLS certificates | `latest().tls.cert_path` | ❌ needs listener restart |
| Secret keys | `latest().auth.jwt_secret` | ⚠️ use with rotation logic |

## Best Practices

### 1. Read `latest()` on every use

Don't cache the config value — read it each time:

```rust
// ✅ GOOD: reads latest on every call
fn max_requests(&self) -> u64 {
    self.config.latest().rate_limit.max_requests
}

// ❌ BAD: only sees the value at construction time
fn max_requests(&self) -> u64 {
    self.cached_max_requests  // stale after config reload
}
```

### 2. Keep `Reloadable<T>` small

Only put **config values** in `Reloadable<T>`, not large data:

```rust
// ✅ GOOD: lightweight config
Reloadable<AppConfig>

// ❌ BAD: large data that changes rarely
Reloadable<Vec<User>>  // use a database query instead
```

### 3. Use `Arc` for expensive clones

If your config contains large fields, wrap them in `Arc`:

```rust
#[derive(Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,      // small, cheap to clone
    pub rate_limit: RateLimitConfig,    // small
    pub allowed_origins: Arc<Vec<String>>,  // large, wrap in Arc
}
```

### 4. Log configuration changes

Always log when config changes — helps debug production issues:

```rust
tx.send(new_config).ok();
tracing::info!(
    pool_size = %new_config.database.pool_size,
    "configuration reloaded"
);
```

## Limitations

| Limitation | Explanation | Workaround |
|-----------|-------------|------------|
| Only `latest()` sees updates | Code that copied the value before the update won't see new values | Call `latest()` on every use |
| No schema validation on reload | Invalid config files propagate errors | Validate before `tx.send()` |
| Watch channel buffer = 1 | Fast reloads may skip intermediate values | OK for config — only latest matters |
| Not for connection strings | Changing DB URL mid-flight doesn't reconnect existing pools | Use connection pool with reconnection logic |

## What you learned

- [x] `Reloadable<T>` provides live config updates without restart
- [x] Use with `ConfigurationLoader` + `watch()` for file watching
- [x] Combine with `Container::with_override()` for full reconfiguration
- [x] Read `latest()` on every use — don't cache the value
- [x] Best for pool sizes, rate limits, feature flags — not connection strings
