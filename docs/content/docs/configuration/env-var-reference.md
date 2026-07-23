---
title: Env-var Reference
description: Complete reference of environment variables supported by Ironic.
---

# Env-var Reference

Ironic and its integrations recognize several environment variables for configuration and runtime behavior.

## Ironic

| Variable | Default | Description |
|----------|---------|-------------|
| `IRONIC_ENV` | — | Active environment profile (`development`, `production`, `staging`) |
| `APP_ENV` | — | Fallback for `IRONIC_ENV` if not set |

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | HTTP server port |
| `HOST` | `0.0.0.0` | HTTP server bind address |
| `SHUTDOWN_TIMEOUT_SECS` | `30` | Graceful shutdown timeout |

## Database

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | PostgreSQL / MySQL / SQLite connection string |
| `DB_POOL_MIN` | `2` | Minimum connection pool size |
| `DB_POOL_MAX` | `10` | Maximum connection pool size |
| `DB_TIMEOUT_SECS` | `30` | Query timeout in seconds |

## Redis

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | `redis://localhost:6379` | Redis connection URL |
| `REDIS_POOL_SIZE` | `5` | Redis connection pool size |

## Auth

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` | — | JWT signing secret (min 32 characters) |
| `JWT_EXPIRY_SECS` | `3600` | Access token TTL in seconds |
| `REFRESH_EXPIRY_SECS` | `86400` | Refresh token TTL in seconds |

## Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `LOG_LEVEL` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |
| `LOG_FORMAT` | `json` | Log format: `json`, `text` |
| `LOG_DIR` | `.logs` | Directory for log files (when using file storage) |

## Metrics

| Variable | Default | Description |
|----------|---------|-------------|
| `METRICS_ENABLED` | `true` | Enable metrics collection |
| `METRICS_PORT` | — | Separate port for metrics endpoint (if different from main port) |

## Feature toggles

Feature flags can be toggled via config, not env vars directly:

```toml
[features]
new_checkout = true
dark_mode = false
```

## Custom env vars

For application-specific env vars, use the standard pattern:

```bash
export APP__CUSTOM_KEY=value
```

Then access via `ConfigurationLoader::environment("APP")`.

## Deprecated variables

| Variable | Replaced by | Deprecated since |
|----------|-------------|-----------------|
| `NODE_ENV` | `IRONIC_ENV` | 0.3.0 |
| `RUST_LOG` | `LOG_LEVEL` | 0.3.0 |
