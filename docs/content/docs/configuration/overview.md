---
title: Configuration Overview
description: Ironic's typed configuration system — files, environment variables, secrets, and hot-reload.
---

# Configuration Overview

Ironic's configuration system loads settings from multiple sources — files, environment variables, JSON layers — and deserializes them into a typed struct with validation.

## How it works

```
config.toml
config.development.toml
environment variables
JSON layers
     │
     ▼
ConfigurationLoader
     │
     ▼
YourAppConfig (typed, validated)
```

## Quick start

### 1. Define your config struct

```rust
use ironic::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
}
```

### 2. Load it

```rust
let config: AppConfig = ConfigurationLoader::new()
    .file("config.toml")
    .auto_detect_env()
    .load()?;
```

### 3. Use it

```rust
println!("Server running on port {}", config.port);
```

## Configuration sources

Sources are layered with increasing precedence:

| Source | Example | Precedence |
|--------|---------|------------|
| Base file | `config.toml` | Lowest |
| Profile overlay | `config.development.toml` | Medium |
| JSON layers | `json(r#"{"port":3000}"#)` | High |
| Environment variables | `APP__PORT=3000` | Highest |

## Validation

Implement `ValidateConfiguration` for custom validation:

```rust
impl ValidateConfiguration for AppConfig {
    fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("port must be > 0".into());
        }
        if self.database_url.is_empty() {
            return Err("database_url is required".into());
        }
        Ok(())
    }
}
```

## Secrets

The `Secret<T>` wrapper redacts sensitive values:

```rust
pub struct AppConfig {
    pub api_key: SecretString,  // displays as [REDACTED]
    pub database_url: SecretString,
}

// Safe logging
info!("API key: {:?}", config.api_key);  // "[REDACTED]"

// Explicit exposure
let key = config.api_key.expose_secret();
```

## Hot reload

With the `hot-reload` feature, config watches files for changes:

```rust
let watcher: ConfigWatcher<AppConfig> = ConfigurationLoader::new()
    .file("config.toml")
    .watch();

// Poll for updates
if let Some(new_config) = watcher.latest() {
    apply_new_config(new_config);
}
```

## Next steps

- [The .env Cascade](/docs/configuration/env-cascade) — How sources merge
- [Alternative Sources](/docs/configuration/alternative-sources) — JSON, env vars, inline
- [Overriding in Tests](/docs/configuration/overriding-in-tests) — Test-specific config
- [Advanced: from_env](/docs/configuration/advanced-from-env) — Manual env parsing
- [Env-var Reference](/docs/configuration/env-var-reference) — All supported variables
