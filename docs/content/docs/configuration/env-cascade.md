---
title: The .env Cascade
description: How configuration sources merge — the cascade from low to high precedence.
---

# The .env Cascade

Ironic merges configuration sources in a specific order. Each source overrides the previous one, giving you fine-grained control over settings per environment.

## Cascade order

```
Lowest precedence
        │
        ▼
  ┌───────────────┐
  │  Base files   │  ← config.toml, config.json (added via .file())
  ├───────────────┤
  │  Profiles     │  ← config.development.toml, config.production.toml
  ├───────────────┤
  │  JSON layers  │  ← Added via .json() method
  ├───────────────┤
  │  Env vars     │  ← APP__PORT=3000 (added via .environment())
  └───────────────┘
        │
        ▼
Highest precedence
```

## How merging works

Values from higher-precedence sources **completely overwrite** values from lower-precedence sources at the key level. Merging is not deep — a key in an env var replaces the entire value from a file.

### Example

Given `config.toml`:
```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "postgres://localhost/mydb"
```

And `config.production.toml`:
```toml
[server]
port = 8080

[database]
url = "postgres://prod/mydb"
```

The merged result is:
```toml
[server]
host = "127.0.0.1"    # from base
port = 8080            # overridden by profile

[database]
url = "postgres://prod/mydb"  # overridden by profile
```

## Environment variable override

With prefix `APP`:
```bash
export APP__SERVER__PORT=443
```

Now port becomes `443`, overriding both files.

## The cascade in code

```rust
let config: AppConfig = ConfigurationLoader::new()
    .file("config.toml")              // lowest priority
    .auto_detect_env()                // config.{env}.toml overlay
    .json(r#"{"server": {"port": 9090}}"#)  // inline override
    .environment("APP")               // highest priority
    .load()?;
```

## Profile auto-detection

The active profile is auto-detected from:

```bash
export IRONIC_ENV=production
# or
export APP_ENV=staging
```

If neither is set, defaults to `"development"`.

The profile file `config.{env}.toml` is loaded as an **optional overlay** — it's silently skipped if it doesn't exist.

## Best practices

- **Base file**: Shared defaults that work in all environments
- **Profile file**: Environment-specific overrides
- **JSON layers**: Test-specific or deploy-time overrides
- **Env vars**: Secrets and deployment-specific values (never commit to version control)
