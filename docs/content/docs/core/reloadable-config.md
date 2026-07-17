---
title: Hot-Reload Config
description: Inject runtime-updating configuration values into providers with Reloadable<T>.
---

# Hot-Reload Config Injection

## What is it?

`Reloadable<T>` wraps a `tokio::sync::watch::Receiver<T>` so providers can read the latest config value without restarting. When the config file changes, the value updates automatically.

## How to use

```rust
use ironic::prelude::*;

#[derive(Injectable)]
struct MyService {
    config: Reloadable<AppConfig>,
}

impl MyService {
    fn get_pool_size(&self) -> u32 {
        self.config.latest().database.pool_size
    }
}
```

## Setup

```rust
let loader = ConfigurationLoader::new()
    .file("config.json");

let config = loader.load::<AppConfig>()?;

// Pass the watch receiver to Reloadable
let (tx, rx) = tokio::sync::watch::channel(config);
let reloadable = Reloadable::new(rx);

// Register as a DI value
let provider = ProviderDefinition::value(reloadable);
container_builder.register(provider);
```

## API

| Method | Returns | Description |
|--------|---------|-------------|
| `latest()` | `T` | Most recent config value (cheap clone) |
| `receiver()` | `watch::Receiver<T>` | Raw receiver for custom watching |

## When to use

| Scenario | Use Reloadable<T> |
|----------|------------------|
| Database pool size changes at runtime | `config.latest().database.pool_size` |
| Feature flags that change during operation | `config.latest().features.enable_cache` |
| API rate limits adjusted without restart | `config.latest().rate_limit.max` |

## Limitation

Only values read through `.latest()` are updated. Code that copied the config before the update won't see new values. Design your providers to call `.latest()` on each use.
