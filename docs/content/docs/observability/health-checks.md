---
title: Health Checks
description: Built-in health endpoint, custom health indicators, and Docker HEALTHCHECK integration.
---

# Health Checks

## What you'll learn

- Use the built-in `GET /health`, `GET /health/live`, and `GET /health/ready` endpoints
- Add custom health indicators with the `HealthIndicator` trait
- Liveness vs. readiness probe distinction
- Composite health checks that aggregate multiple indicators
- Configurable check timeouts
- IntegrationHealth for databases, Redis, and external services
- Understand health check aggregation
- Integrate health checks with Docker HEALTHCHECK

---

## HealthModule

Import `HealthModule` to get health and version endpoints out of the box:

```rust
use ironic::HealthModule;

#[derive(Module)]
#[module(imports = [HealthModule])]
struct AppModule;
```

### Endpoints

| Endpoint | Purpose | Response |
|----------|---------|----------|
| `GET /health` | Composite health (readiness, backward compatible) | `{"status": "ok"}` — 200/207/503 |
| `GET /health/live` | Liveness probe (process alive) | `{"status": "alive"}` — always 200 |
| `GET /health/ready` | Readiness probe (dependencies healthy) | `{"status": "ok"}` — 200/503 |
| `GET /version` | Build metadata | `{"git_sha": "abc123", ...}` — 200 |

### Composite health (`GET /health`)

Aggregates all registered `HealthIndicator::check()` results. HTTP status: `200` when
healthy, `207 Multi-Status` when degraded, `503` when unhealthy.

### Liveness probe (`GET /health/live`)

Returns `200 OK` with `{"status": "alive"}` without invoking any dependency checks.
Use this for Kubernetes `livenessProbe` to know when the process should be restarted.

### Readiness probe (`GET /health/ready`)

Aggregates all registered `HealthIndicator::check_readiness()` results. Returns `200 OK`
when all dependencies are healthy, `503 Service Unavailable` when any dependency is
degraded or unhealthy. Use this for Kubernetes `readinessProbe` to know when the
process should receive traffic.

## Composite health

When multiple health indicators are registered, the endpoint returns an
aggregated status and per-component detail:

```json
{
  "status": "degraded",
  "checks": {
    "database": "ok",
    "redis": "ok",
    "external_api": "unhealthy"
  }
}
```

### HTTP status codes

| Indicator states | HTTP status |
|------------------|-------------|
| All ok | `200 OK` |
| Any degraded | `207 Multi-Status` |
| Any unhealthy | `503 Service Unavailable` |

## HealthIndicator trait

```rust
use std::pin::Pin;
use std::future::Future;
use ironic::{HealthIndicator, HealthStatus};

pub trait HealthIndicator: Send + Sync {
    /// Name of this component, shown in the health response.
    fn name(&self) -> &str;

    /// Runs a health check (deprecated, use `check_readiness` instead).
    #[deprecated(since = "0.5.0", note = "use `check_readiness` instead")]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>>;

    /// Reports whether the component's process is alive.
    /// Default: always returns `HealthStatus::Ok`.
    fn check_liveness(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(std::future::ready(HealthStatus::Ok))
    }

    /// Reports whether the component is ready to serve traffic.
    /// Default: delegates to `check()` for backward compatibility.
    fn check_readiness(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        #[allow(deprecated)]
        self.check()
    }
}
```

Every indicator runs in parallel — a slow database check does not delay the
Redis check.  If an indicator exceeds the configured timeout, it is reported
as `unhealthy`.

### Liveness vs. Readiness

The `HealthIndicator` trait distinguishes between two probe types:

- **Liveness** (`check_liveness`): Is the process itself alive? Defaults to `Ok`
  for all indicators. Override only if your component can detect fatal internal
  state (e.g., poisoned connection pool, corrupted in-memory cache).

- **Readiness** (`check_readiness`): Is the component ready to serve traffic?
  Defaults to calling the existing `check()` for backward compatibility.
  Override to implement dependency-aware health logic (e.g., database reachable,
  upstream API responsive).

The existing `check()` method is **deprecated** since v0.5.0. New code should
implement `check_readiness()` instead. The default implementation of
`check_readiness()` delegates to `check()`, so existing indicators continue to
work without changes.

## HealthStatus variants

```rust
pub enum HealthStatus {
    /// Component is functioning correctly.
    Ok,
    /// Component is working but degraded (e.g., high latency, reduced capacity).
    Degraded { message: Option<String> },
    /// Component is not working (e.g., connection refused, timeout).
    Unhealthy { error: String },
}
```

## Real-world health indicators

### PostgreSQL database

```rust
use std::pin::Pin;
use std::future::Future;
use sqlx::PgPool;
use ironic::{HealthIndicator, HealthStatus};

struct DatabaseHealth {
    pool: PgPool,
}

impl HealthIndicator for DatabaseHealth {
    fn name(&self) -> &str { "database" }

    #[allow(deprecated)]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            match sqlx::query("SELECT 1").execute(&self.pool).await {
                Ok(_) => HealthStatus::Ok,
                Err(e) => HealthStatus::Unhealthy { error: format!("PostgreSQL query failed: {e}") },
            }
        })
    }
}
```

### Redis cache

```rust
use std::pin::Pin;
use std::future::Future;
use redis::aio::ConnectionManager;
use ironic::{HealthIndicator, HealthStatus};

struct RedisHealth {
    conn: ConnectionManager,
}

impl HealthIndicator for RedisHealth {
    fn name(&self) -> &str { "redis" }

    #[allow(deprecated)]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            match redis::cmd("PING").query_async::<String>(&self.conn.clone()).await {
                Ok(ref reply) if reply == "PONG" => HealthStatus::Ok,
                Ok(reply) => HealthStatus::Degraded { message: Some(format!("Unexpected PING response: {reply}")) },
                Err(e) => HealthStatus::Unhealthy { error: format!("Redis connection failed: {e}") },
            }
        })
    }
}
```

### External HTTP API

```rust
use std::pin::Pin;
use std::future::Future;
use std::time::Duration;
use ironic::{HealthIndicator, HealthStatus};

struct ExternalApiHealth {
    url: String,
    client: reqwest::Client,
}

impl HealthIndicator for ExternalApiHealth {
    fn name(&self) -> &str { "payment_api" }

    #[allow(deprecated)]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            match self.client.get(&self.url).timeout(Duration::from_secs(5)).send().await {
                Ok(resp) if resp.status().is_success() => HealthStatus::Ok,
                Ok(resp) => HealthStatus::Degraded { message: Some(format!("API returned {}", resp.status())) },
                Err(e) => HealthStatus::Unhealthy { error: format!("API unreachable: {e}") },
            }
        })
    }
}
```

### Disk space

```rust
use std::pin::Pin;
use std::future::Future;
use ironic::{HealthIndicator, HealthStatus};

struct DiskHealth {
    path: &'static str,
    min_free_bytes: u64,
}

impl HealthIndicator for DiskHealth {
    fn name(&self) -> &str { "disk" }

    #[allow(deprecated)]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            let free = tokio::task::spawn_blocking(move || {
                std::fs::metadata(self.path)
                    .and_then(|_| /* platform-specific free space check */ Ok(0u64))
            }).await.unwrap_or(0);

            if free >= self.min_free_bytes {
                HealthStatus::Ok
            } else {
                HealthStatus::Degraded { message: Some(format!("Low disk space: {free} bytes free")) }
            }
        })
    }
}
```

## Custom health indicators

Register indicators for each dependency your app needs:

```rust
use std::pin::Pin;
use std::future::Future;
use ironic::{HealthIndicator, HealthStatus};
use ironic::Inject;

struct DatabaseHealth {
    pool: Inject<DbPool>,
}

impl HealthIndicator for DatabaseHealth {
    fn name(&self) -> &str {
        "database"
    }

    #[allow(deprecated)]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            match self.pool.acquire().await {
                Ok(_) => HealthStatus::Ok,
                Err(e) => HealthStatus::Unhealthy { error: e.to_string() },
            }
        })
    }
}
```

Register indicators in your module:

```rust
use std::sync::Arc;
use ironic::HealthModule;

#[ironic::module(imports = [HealthModule])]
struct AppModule;

// Register indicators at startup
ironic::register(Arc::new(DatabaseHealth::new()));
ironic::register(Arc::new(RedisHealth::new()));
ironic::register(Arc::new(ExternalApiHealth::new()));
```

Response with custom indicators:

```json
{
  "status": "ok",
  "components": {
    "database": {"status": "up"},
    "redis": {"status": "up"},
    "external-api": {"status": "down", "detail": {"error": "connection refused"}}
  }
}
```

## IntegrationHealth wiring

Database and Redis integrations automatically provide health indicators when
their feature flags are enabled.  No manual wiring is needed — the integration
modules call `register()` at startup.

| Integration | Feature flag | Indicator name |
|-------------|-------------|----------------|
| SQLx | `sqlx` / `sqlx-*` | `database` |
| SeaORM | `seaorm` / `seaorm-*` | `database` |
| Diesel | `diesel` | `database` |
| MongoDB | `mongodb` | `mongodb` |
| Redis | `redis` | `redis` |

## Configurable timeout

Health checks are subject to a configurable timeout (default 5 seconds).
Indicators that exceed the timeout are reported as `unhealthy` rather than
blocking the health endpoint.

```rust
use ironic::HealthConfig;
use std::time::Duration;

ironic::health::configure(HealthConfig {
    check_timeout: Duration::from_secs(3),
});
```

## Health check aggregation

The overall status is determined by the **worst** status across all indicators:

| All indicators | Overall status | HTTP status |
|---|---|---|
| All `Ok` | `ok` | `200 OK` |
| At least one `Degraded`, none `Unhealthy` | `degraded` | `207 Multi-Status` |
| At least one `Unhealthy` | `unhealthy` | `503 Service Unavailable` |

### Aggregation algorithm

```rust
fn aggregate(results: &[HealthStatus]) -> (&'static str, u16) {
    let mut any_degraded = false;
    for status in results {
        match status {
            HealthStatus::Unhealthy { .. } => return ("unhealthy", 503),
            HealthStatus::Degraded { .. } => any_degraded = true,
            HealthStatus::Ok => {}
        }
    }
    if any_degraded { ("degraded", 207) } else { ("ok", 200) }
}
```

## Testing health indicators

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::future::Future;

    #[tokio::test]
    async fn test_database_health_ok() {
        let pool = PgPool::connect("postgres://localhost/testdb").await.unwrap();
        let indicator = DatabaseHealth { pool };
        #[allow(deprecated)]
        let status = indicator.check().await;
        assert!(matches!(status, HealthStatus::Ok));
    }

    #[tokio::test]
    async fn test_health_indicator_ok() {
        let indicator = MockIndicator { name: "db", status: HealthStatus::Ok };
        #[allow(deprecated)]
        let status = indicator.check().await;
        assert!(matches!(status, HealthStatus::Ok));
    }

    #[tokio::test]
    async fn test_health_indicator_unhealthy() {
        let indicator = MockIndicator {
            name: "redis",
            status: HealthStatus::Unhealthy { error: "connection refused".into() },
        };
        #[allow(deprecated)]
        let status = indicator.check().await;
        assert!(matches!(status, HealthStatus::Unhealthy { .. }));
    }
}

struct MockIndicator {
    name: &'static str,
    status: HealthStatus,
}

impl HealthIndicator for MockIndicator {
    fn name(&self) -> &str { self.name }
    #[allow(deprecated)]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(std::future::ready(self.status.clone()))
    }
}
```

## Docker HEALTHCHECK

```dockerfile
FROM rust:1.80-slim AS builder
# ... build steps ...

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-api /usr/local/bin/my-api
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

CMD ["/usr/local/bin/my-api"]
```

Docker will poll `/health` every 15 seconds and restart the container if it fails 3 consecutive checks.

For Kubernetes deployments, use the dedicated probe endpoints:

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 3000
readinessProbe:
  httpGet:
    path: /health/ready
    port: 3000
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Indicator blocks the async runtime | Avoid `.unwrap()` or blocking I/O inside `check()` |
| Forgetting to register indicators | Call `register()` at startup |
| Health check is slow | Keep `check()` under 1 second. Use timeouts for external calls |
| Not importing `HealthModule` | Add `HealthModule` to the module's `imports` |

## What you learned

- [x] `HealthModule` provides `GET /health`, `GET /health/live`, `GET /health/ready`, and `GET /version`
- [x] Liveness probes (`/health/live`) report process health without dependency checks
- [x] Readiness probes (`/health/ready`) aggregate dependency readiness for traffic routing
- [x] `HealthIndicator` trait defines `check_liveness()` and `check_readiness()` with defaults
- [x] The existing `check()` method is deprecated — new code should use `check_readiness()`
- [x] Composite health returns `200` / `207` / `503` based on aggregate status
- [x] IntegrationHealth wiring auto-registers database/Redis indicators
- [x] Configurable timeout prevents slow checks from blocking the endpoint
- [x] Docker HEALTHCHECK and Kubernetes probes integrate directly with the health endpoints
