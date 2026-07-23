---
title: Alternative Sources
description: Load configuration from JSON, environment variables, and inline strings.
---

# Alternative Sources

Beyond TOML files, Ironic supports several alternative configuration sources.

## JSON layers

Inline JSON strings can override file-based config:

```rust
let config: AppConfig = ConfigurationLoader::new()
    .file("config.toml")
    .json(r#"{
        "server": {
            "port": 4000
        }
    }"#)
    .load()?;
```

JSON layers have higher precedence than files but lower than environment variables.

## Environment variables

Environment variables use `__` as a separator for nested keys:

```rust
let config: AppConfig = ConfigurationLoader::new()
    .environment("APP")  // prefix
    .load()?;
```

With prefix `APP`:

| Environment variable | Config key |
|---------------------|------------|
| `APP__PORT=3000` | `port = 3000` |
| `APP__SERVER__HOST=0.0.0.0` | `server.host = "0.0.0.0"` |
| `APP__DATABASE__URL=postgres://localhost/db` | `database.url = "postgres://localhost/db"` |

Environment variables have the **highest precedence** — they override all other sources.

## Multiple prefixes

```rust
ConfigurationLoader::new()
    .environment("APP")     // APP__PORT=3000
    .environment("MYAPP")   // MYAPP__PORT=4000
```

## TOML files

```rust
ConfigurationLoader::new()
    .file("config.toml")          // required
    .file("/etc/app/config.toml")  // system-wide override
```

Files added later have higher precedence.

## Inline TOML

```rust
ConfigurationLoader::new()
    .json(r#"{"port": 3000}"#)
```

Note: the method is named `.json()` but accepts any format supported by the `config` crate (JSON, TOML, YAML via features).

## Custom sources

For custom sources, use the underlying `config` crate builder:

```rust
use config::{Config, File, Environment};

let builder = Config::builder()
    .add_source(File::with_name("config"))
    .add_source(Environment::with_prefix("APP"));
```
