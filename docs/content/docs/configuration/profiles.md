---
title: Configuration Profiles
description: Manage environment-specific settings with config profiles — development, staging, production.
---

# Configuration Profiles

## What you'll learn

- How environment profiles let you override settings per deployment
- File precedence: env vars > `config.{env}.toml` > `config.toml`
- Auto-detect the active profile from environment variables
- Real-world example with a complete three-tier config setup

---

## How profiles work

The `ConfigurationLoader` loads base config files then overlays a profile-specific
file (`config.{env}.toml`) on top.  Values in the profile file **override** the
base, and environment variables override both.

```
Precedence  ──  highest
  env vars       ↑
  config.{env}.toml
  config.toml    ↓
             ──  lowest
```

## Real-world example: three-tier config

### 1. Base config (`config.toml`) — shared defaults

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "postgres://localhost:5432/myapp"
max_connections = 10

[logging]
level = "info"

[features]
new_checkout = false
dark_mode = false
```

### 2. Development overrides (`config.development.toml`) — local dev

```toml
[server]
host = "127.0.0.1"
port = 3000

[logging]
level = "debug"

[features]
dark_mode = true
```

### 3. Production overrides (`config.production.toml`) — live

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
max_connections = 100

[logging]
level = "warn"

[features]
new_checkout = true
```

### 4. Rust config structs

```rust
use serde::Deserialize;
use ironic::{ConfigurationLoader, ValidateConfiguration};

#[derive(Debug, Deserialize)]
struct AppConfig {
    server: ServerConfig,
    database: DatabaseConfig,
    logging: LoggingConfig,
    features: std::collections::HashMap<String, bool>,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    url: String,
    max_connections: u32,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
}

impl ValidateConfiguration for AppConfig {
    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("server.port must be greater than 0".into());
        }
        if self.database.max_connections == 0 {
            return Err("database.max_connections must be greater than 0".into());
        }
        Ok(())
    }
}
```

### 5. Loading with auto-detection

```rust
use ironic::ConfigurationLoader;

fn main() -> Result<(), ironic::ConfigurationError> {
    // IRONIC_ENV=production → loads config.toml + config.production.toml
    // IRONIC_ENV unset      → loads config.toml + config.development.toml
    let config: AppConfig = ConfigurationLoader::new()
        .file("config.toml")
        .auto_detect_env()
        .load()?;

    println!("Listening on {}:{}", config.server.host, config.server.port);
    Ok(())
}
```

### 6. Loading with explicit profile

```rust
let config: AppConfig = ConfigurationLoader::new()
    .file("config.toml")
    .profile("staging")       // loads config.staging.toml
    .environment("APP")       // APP__SERVER__PORT=4000 overrides the file
    .load()?;
```

## Environment variable override

Environment variables take the highest precedence.  Use a prefix with `__` as
the separator for nested keys:

```bash
# Override any config value at runtime
APP__SERVER__PORT=4000
APP__DATABASE__MAX_CONNECTIONS=200
APP__FEATURES__NEW_CHECKOUT=true
```

This lets you override values in CI/CD without changing config files:

```rust
let config: AppConfig = ConfigurationLoader::new()
    .file("config.toml")
    .auto_detect_env()
    .environment("APP")       // env vars override everything
    .load()?;
```

## Testing with profiles

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_profile() {
        std::env::set_var("IRONIC_ENV", "development");
        let config: AppConfig = ConfigurationLoader::new()
            .file("config.toml")
            .auto_detect_env()
            .load()
            .expect("development config should load");
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_profile_overrides_base() {
        let config: AppConfig = ConfigurationLoader::new()
            .json(r#"{"server":{"host":"127.0.0.1","port":3000},"database":{"url":"postgres://localhost/db","max_connections":10},"logging":{"level":"info"}}"#)
            .profile("staging")
            .json(r#"{"server":{"port":9090}}"#)
            .load()
            .expect("json should override profile");
        assert_eq!(config.server.port, 9090);
    }
}
```

## Example layout

```
config.toml              ← shared defaults
config.development.toml  ← local overrides (git-ignored)
config.staging.toml      ← staging overrides
config.production.toml   ← production overrides
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Profile file silently ignored | File must be named `config.{env}.toml` verbatim |
| Env var not picked up | Set `IRONIC_ENV=production` not `IRONIC_ENV=config.production.toml` |
| Profile not applied | Call `auto_detect_env()` or `profile()` **before** `load()` |
| Missing validation | Implement `ValidateConfiguration` — deserialization alone catches typos but not semantic errors |
| Secret in config file | Use `SecretString` in your config struct and set the value via env var |

## What you learned

- [x] `auto_detect_env()` reads `IRONIC_ENV` or `APP_ENV` with `"development"` fallback
- [x] `profile("env")` loads `config.{env}.toml` as an optional overlay
- [x] Precedence: env vars > profile file > base config files
- [x] Missing profile files are silently skipped (no error)
- [x] `ValidateConfiguration` catches semantic errors after deserialization
- [x] Environment variables use `PREFIX__KEY__NESTED` syntax for deep overrides
