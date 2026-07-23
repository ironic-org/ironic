---
title: "Advanced: Hand-written from_env"
description: Manually parse environment variables into configuration structs.
---

# Advanced: Hand-written from_env

For scenarios where you need full control over environment variable parsing — or you want to avoid the `config` crate entirely — you can write your own `from_env` function.

## When to use this

- You want **zero dependencies** beyond `serde`
- Your config is **simple** (a few env vars, no nesting)
- You need **custom parsing logic** for specific variables
- You want to **avoid file I/O** entirely

## Manual from_env pattern

```rust
use std::env;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
    pub redis_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .map_err(|e| ConfigError::ParseError {
                    key: "PORT".into(),
                    value: env::var("PORT").unwrap_or_default(),
                    source: e.to_string(),
                })?,

            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingKey {
                    key: "DATABASE_URL".into(),
                })?,

            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".into()),

            redis_url: env::var("REDIS_URL").ok(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {key}")]
    MissingKey { key: String },

    #[error("Failed to parse {key}={value}: {source}")]
    ParseError { key: String, value: String, source: String },
}
```

## Using with ConfigurationLoader

You can combine manual parsing with `ConfigurationLoader`:

```rust
let config = ConfigurationLoader::new()
    .file("config.toml")
    .load::<AppConfig>()  // tries files + env vars via config crate
    .unwrap_or_else(|_| AppConfig::from_env().unwrap());  // fallback to env
```

## When to avoid

Don't use `from_env` when:

- You have **nested config** (server.host, database.url, etc.)
- You need **profile support** (dev/staging/prod)
- You want **hot-reload** functionality
- You need **validation** beyond type parsing
- You have **many configuration values** (>10)

For these cases, use `ConfigurationLoader` with the standard layered approach.
