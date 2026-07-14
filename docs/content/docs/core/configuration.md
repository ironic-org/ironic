---
title: Configuration
description: Load settings from files, environment variables, and keep secrets safe with Ironic's typed configuration system.
---

# Configuration

## What you'll learn

- Load settings from `ironic.toml` and environment variables
- Use typed configuration structs with validation
- Keep secrets (passwords, API keys) safe from logs
- Override settings per environment (dev, staging, production)

## The big picture

Your app needs settings — database URLs, API keys, feature flags. Ironic loads them all into a **typed struct** so you get compile-time safety:

```
ironic.toml + env vars  ──►  ConfigurationLoader  ──►  YourAppConfig { ... }
   (files)                      (framework)               (typed struct)
```

## Step 1: Define your config

Create a struct with `#[derive(ValidateConfiguration)]`:

```rust
use ironic::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, ValidateConfiguration)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: Option<String>,    // ← Optional: can be missing
    pub port: u16,
    pub log_level: String,
}
```

> **Why typed?** If you misspell `database_url` in your config file, the framework tells you exactly what's wrong — at startup, not at 3 AM in production.

## Step 2: Create `ironic.toml`

This file lives next to your `Cargo.toml`:

```toml
[project]
name = "my-api"
source_root = "src"
default_module = "src/app.rs"

[settings]
database_url = "postgres://localhost:5432/mydb"
port = 3000
log_level = "info"
```

## Step 3: Load it at startup

```rust
#[ironic::main]
async fn main() {
    let config: AppConfig = ConfigurationLoader::new()
        .load()
        .expect("Failed to load configuration");

    println!("Starting on port {}", config.port);
    println!("Database: {}", config.database_url);

    // Pass config to your services...
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build().await.unwrap()
        .listen(format!("127.0.0.1:{}", config.port)).await.unwrap();
}
```

## Keeping secrets safe

Never log passwords or API keys. Use `Secret<T>` to automatically redact them:

```rust
use ironic::Secret;

#[derive(Debug, Clone, Deserialize, ValidateConfiguration)]
pub struct AppConfig {
    pub api_key: Secret<String>,     // ← Wrapped in Secret<T>
    pub db_password: Secret<String>,
}

// When printed: "api_key: [REDACTED]"
// In logs:      "db_password: [REDACTED]"
// When used:    config.api_key.as_ref() → returns &str
```

> `Secret<T>` redacts itself in Debug, Display, and serialization output — no more accidentally leaking secrets in logs!

## Environment variables

Settings can be overridden with environment variables:

```bash
# Override settings from ironic.toml
DATABASE_URL="postgres://prod:5432/db" cargo run
PORT=8080 cargo run
```

The env var name matches the field name (uppercased). Environment variables **always win** over file settings.

## Configuration sources (priority order)

```
1. Environment variables     ← Highest priority
2. ironic.toml              ← File settings
3. Default values            ← Serde #[serde(default)]
```

## Multiple environments

For different environments, create separate config files:

```
ironic.toml              ← Default (development)
ironic.production.toml   ← Production overrides
```

Load based on an environment variable:

```rust
let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
let config: AppConfig = ConfigurationLoader::new()
    .with_file(format!("ironic.{env}.toml"))
    .load()
    .expect("Failed to load config");
```

## Try it yourself

1. Add a `port` field to a config struct
2. Set it to `3000` in `ironic.toml`
3. Override it with `PORT=8080 cargo run`
4. Verify the app listens on port 8080

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Field name mismatch in toml | TOML uses `snake_case`, Rust struct uses `snake_case` — they must match exactly |
| Secret logged by accident | Always use `Secret<T>`, never `String` for sensitive values |
| Missing required field | Add a default with `#[serde(default = "value")]` or provide it in every config file |
| Wrong types | `port: u16` in Rust but `port = "3000"` (string) in TOML → use `port = 3000` (number) |

## What you learned

- [x] Define typed config structs with `#[derive(ValidateConfiguration)]`
- [x] Create `ironic.toml` for file-based settings
- [x] Load config with `ConfigurationLoader`
- [x] Protect secrets with `Secret<T>`
- [x] Override settings with environment variables
- [x] Handle multiple environments

## Next steps

Learn how Dependency Injection connects services together:

→ [Dependency Management](./dependency-management)
