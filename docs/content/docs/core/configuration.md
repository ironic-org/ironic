---
title: Configuration
description: Load typed settings, validate startup state, and redact secrets.
---

# Typed configuration

`ConfigurationLoader` layers TOML files, JSON, and prefixed environment variables before
deserializing one application-owned type.

```rust
use ironic::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {
    port: u16,
    database_password: SecretString,
}

impl ValidateConfiguration for Settings {
    fn validate(&self) -> Result<(), String> {
        (self.port > 0)
            .then_some(())
            .ok_or_else(|| "port must be greater than zero".to_owned())
    }
}

let settings = ConfigurationLoader::new()
    .file("config.toml")
    .environment("APP")
    .load::<Settings>()?;
# Ok::<(), ConfigurationError>(())
```

Environment nesting uses `__`: `APP__SERVER__PORT=3000`. `Secret<T>` values deserialize normally
but render as `[REDACTED]` through `Debug`, `Display`, and serialization. Only
`expose_secret()` deliberately reveals the underlying value.
