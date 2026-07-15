---
title: "Layered Configuration — TOML, JSON, env vars, and secret redaction"
description: "A deep dive into Ironic's typed, validated configuration layer: how ConfigurationLoader merges sources, the __ nested-key convention, custom validation, and the Secret<T> wrapper that prevents credential leaks."
date: "2026-07-15"
author: "Ironic Team"
---

# Layered Configuration — TOML, JSON, env vars, and secret redaction

Most Rust frameworks settle for `figment` or an ad-hoc `env::var` call sprinkled across thirty files. Ironic takes a different path: a single, typed configuration surface backed by the `config` crate, with layers that merge deterministically, a domain-level validation contract, and a transparent wrapper that makes accidental credential exposure impossible. The whole thing lives in under 200 lines inside `crates/ironic-config/src/lib.rs`.

---

## The `ConfigurationLoader` builder

Everything starts with `ConfigurationLoader`, a thin builder over `config::ConfigBuilder<DefaultState>` (`lib.rs:34-36`). It exposes three source methods — `file`, `json`, and `environment` — each appending a layer onto the internal builder:

```rust
pub struct ConfigurationLoader {
    builder: config::ConfigBuilder<DefaultState>,
}
```

- **`file(path)`** — adds a required TOML file via `File::from(path).required(true)`. If the file is missing, the loader returns a `ConfigurationError::Source` at build time (`lib.rs:49-54`).
- **`json(source)`** — parses an inline JSON string. Useful for embedding default configuration at compile time or feeding test fixtures without touching the filesystem (`lib.rs:58-63`).
- **`environment(prefix)`** — mounts environment variables under an optional prefix. Internally it sets both the prefix separator and the key separator to `__`, enabling the nested-key convention (`lib.rs:69-77`).

The builder is consumed by a single terminal method: `load::<T>()`. It calls `config::ConfigBuilder::build()`, deserializes into the caller's concrete type, and then runs `T::validate()` (`lib.rs:84-93`). The entire pipeline — missing files, type mismatches, validation failures — funnels through the `ConfigurationError` enum, which carries an error-code prefix (`RF_CONFIG_SOURCE`, `RF_CONFIG_VALIDATION`) for observability.

---

## Merge order and the `__` separator

The `config` crate merges sources in the order they are added, with later sources overwriting earlier keys. A typical Ironic bootstrap looks like this:

```rust
let cfg = ConfigurationLoader::new()
    .file("config/default.toml")    // Layer 1 — lowest priority
    .file("config/overrides.toml")  // Layer 2 — overrides defaults
    .environment("APP")             // Layer 3 — highest priority
    .load::<AppConfig>()?;
```

The `__` separator convention turns `APP__SERVER__PORT=3000` into the nested path `server.port`. The call `.separator("__")` tells the `config` crate to treat double underscores as hierarchy delimiters, and `.try_parsing(true)` ensures that `3000` becomes a `u16` rather than remaining a string — avoiding a separate casting step in application code.

This layering model means developers can ship a complete TOML configuration, override select values with JSON in a deployment manifest, and patch individual keys at runtime via environment variables — all without touching application logic.

---

## `ValidateConfiguration` — custom domain validation

Type-level deserialization catches shape errors (wrong field names, incorrect types), but it cannot enforce domain rules — an `Option<String>` field is still a `String`, and a `u16` port of `0` is still a `u16`. Ironic addresses this with the `ValidateConfiguration` trait (`lib.rs:9-16`):

```rust
pub trait ValidateConfiguration {
    fn validate(&self) -> Result<(), String>;
}
```

The `load::<T>()` method requires `T: ValidateConfiguration`. After deserialization, it calls `validate()` and maps any `Err` into `ConfigurationError::Validation`. This is where rules like port ranges, mutually exclusive fields, or required combinations live:

```rust
impl ValidateConfiguration for AppConfig {
    fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("port must be greater than zero".to_owned());
        }
        Ok(())
    }
}
```

The error message is a plain `String` — safe to display in startup diagnostics — with no raw configuration data accidentally exposed.

---

## `Secret<T>` — redacted by default

The sharpest edge in configuration is credentials. Database URLs, API tokens, and signing keys all pass through the same deserialization pipeline as hostnames and port numbers. A stray `dbg!()` or a misconfigured error reporter can leak them.

Ironic solves this with `Secret<T>` (`lib.rs:103-146`), a `#[serde(transparent)]` newtype that wraps a single inner value. The key property: **every safe output channel is blocked**.

```rust
#[derive(Clone, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct Secret<T>(T);
```

`Debug` prints `Secret([REDACTED])`. `Display` prints `[REDACTED]`. `Serialize` — via a custom impl — serializes as the string `"[REDACTED]"`. There is no `Deref`, no `AsRef`, no `From` impl. The inner value is accessible through exactly two methods:

- `expose_secret(&self) -> &T` — explicit, auditable, grep-friendly.
- `into_secret(self) -> T` — consumes the wrapper, making re-exposure impossible.

Type aliases provide ergonomics for common cases: `SecretString` is `Secret<String>`. A configuration struct uses it naturally:

```rust
pub struct AppConfig {
    pub port: u16,
    pub token: SecretString,
}
```

In tests (`lib.rs:191-196`), `SecretString::new("private")` formats as `[REDACTED]` in both `Display` and `Debug`, and serializes to `"[REDACTED]"` via `serde_json`. Crates like `tracing` will never see the real value unless code explicitly calls `expose_secret()`.

---

## Concrete example: database config with env override

Consider a production database configuration. The TOML file defines sensible defaults:

```toml
# config/default.toml
[database]
host = "localhost"
port = 5432
url = "postgres://user:pass@localhost:5432/db"
```

The corresponding Rust struct uses `SecretString` for the URL:

```rust
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub url: SecretString,
}
```

At deployment time, the operations team sets `APP__DATABASE__URL=postgres://prod-user:secret@pg.internal:5432/prod`. Because the environment layer was added last, the TOML `database.url` is silently replaced. The application code reads the resolved value with `config.database.url.expose_secret()`, and every log statement, error report, and health-check response sees only `[REDACTED]`. If a developer forgets the `expose_secret()` call and tries to pass the `SecretString` directly to a connection pool constructor, the type system rejects it — there is no automatic coercion.

---

## Summary

Ironic's configuration layer is small, composable, and principled. `ConfigurationLoader` provides a single entry point for layered TOML, JSON, and environment sources. `ValidateConfiguration` separates shape errors from domain errors. `Secret<T>` ensures that secrets cannot leak through logging, serialization, or formatting — only through explicit, auditable access. Together they form a pipeline where configuration flows from disk to typed struct without a single `env::var` call scattered across the codebase.
