---
title: Health Checks
description: Built-in health endpoint, custom health indicators, and Docker HEALTHCHECK integration.
---

# Health Checks

## What you'll learn

- Use the built-in `GET /health` endpoint
- Add custom health indicators with the `HealthIndicator` trait
- Composite health checks that aggregate multiple indicators
- Configurable check timeouts
- IntegrationHealth for databases, Redis, and external services
- Understand health check aggregation
- Integrate health checks with Docker HEALTHCHECK

---

## HealthModule

Import `HealthModule` to get a `GET /health` endpoint out of the box:

```rust
use ironic::health::HealthModule;

#[derive(Module)]
#[module(imports = [HealthModule])]
struct AppModule;
```

Response:

```json
{"status": "ok"}
```

HTTP status: `200` when healthy, `503` when degraded.

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
use ironic::health::{HealthIndicator, HealthStatus};
use std::future::Future;

pub trait HealthIndicator: Send + Sync {
    /// Name of this component, shown in the health response.
    fn name(&self) -> &str;

    /// Runs the health check. Called concurrently with other indicators.
    fn check(&self) -> impl Future<Output = HealthStatus> + Send;
}
```

Every indicator runs in parallel — a slow database check does not delay the
Redis check.  If an indicator exceeds the configured timeout, it is reported
as `unhealthy`.

## HealthStatus variants

```rust
pub enum HealthStatus {
    /// Component is functioning correctly.
    Ok,
    /// Component is working but degraded (e.g., high latency, reduced capacity).
    Degraded { message: String },
    /// Component is not working (e.g., connection refused, timeout).
    Unhealthy { error: String },
}
```

### Helper methods

```rust
impl HealthStatus {
    /// Creates an `Ok` status.
    pub fn up() -> Self { Self::Ok }

    /// Creates an `Unhealthy` status.
    pub fn down(error: impl Into<String>) -> Self {
        Self::Unhealthy { error: error.into() }
    }

    /// Attaches a detail message to a `Degraded` or `Unhealthy` status.
    pub fn with_detail(self, key: &str, value: String) -> Self { /* ... */ }
}
```

## Real-world health indicators

### PostgreSQL database

```rust
use sqlx::PgPool;
use ironic::health::{HealthIndicator, HealthStatus};

struct DatabaseHealth {
    pool: PgPool,
}

impl HealthIndicator for DatabaseHealth {
    fn name(&self) -> &str { "database" }

    async fn check(&self) -> HealthStatus {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => HealthStatus::up(),
            Err(e) => HealthStatus::down(format!("PostgreSQL query failed: {e}")),
        }
    }
}
```

### Redis cache

```rust
use redis::aio::ConnectionManager;
use ironic::health::{HealthIndicator, HealthStatus};

struct RedisHealth {
    conn: ConnectionManager,
}

impl HealthIndicator for RedisHealth {
    fn name(&self) -> &str { "redis" }

    async fn check(&self) -> HealthStatus {
        match redis::cmd("PING").query_async::<String>(&self.conn.clone()).await {
            Ok(ref reply) if reply == "PONG" => HealthStatus::up(),
            Ok(reply) => HealthStatus::degraded(format!("Unexpected PING response: {reply}")),
            Err(e) => HealthStatus::down(format!("Redis connection failed: {e}")),
        }
    }
}
```

### External HTTP API

```rust
use ironic::health::{HealthIndicator, HealthStatus};

struct ExternalApiHealth {
    url: String,
    client: reqwest::Client,
}

impl HealthIndicator for ExternalApiHealth {
    fn name(&self) -> &str { "payment_api" }

    async fn check(&self) -> HealthStatus {
        match self.client.get(&self.url).timeout(Duration::from_secs(5)).send().await {
            Ok(resp) if resp.status().is_success() => HealthStatus::up(),
            Ok(resp) => HealthStatus::degraded(format!("API returned {}", resp.status())),
            Err(e) => HealthStatus::down(format!("API unreachable: {e}")),
        }
    }
}
```

### Disk space

```rust
struct DiskHealth {
    path: &'static str,
    min_free_bytes: u64,
}

impl HealthIndicator for DiskHealth {
    fn name(&self) -> &str { "disk" }

    async fn check(&self) -> HealthStatus {
        let free = tokio::task::spawn_blocking(move || {
            std::fs::metadata(self.path)
                .and_then(|_| /* platform-specific free space check */ Ok(0u64))
        }).await.unwrap_or(0);

        if free >= self.min_free_bytes {
            HealthStatus::up()
        } else {
            HealthStatus::degraded(format!("Low disk space: {free} bytes free"))
        }
    }
}
```

## Custom health indicators

Register indicators for each dependency your app needs:

```rust
use ironic::health::{HealthIndicator, HealthStatus};
use ironic::Inject;

struct DatabaseHealth {
    pool: Inject<DbPool>,
}

#[async_trait]
impl HealthIndicator for DatabaseHealth {
    fn name(&self) -> &str {
        "database"
    }

    async fn check(&self) -> HealthStatus {
        match self.pool.acquire().await {
            Ok(_) => HealthStatus::up(),
            Err(e) => HealthStatus::down()
                .with_detail("error", e.to_string()),
        }
    }
}
```

Register indicators in your module:

```rust
use ironic::health::{HealthModule, HealthRegistry};

#[ironic::module(imports = [HealthModule])]
struct AppModule;

impl AppModule {
    fn configure(registry: Inject<HealthRegistry>) {
        registry.register(DatabaseHealth::new());
        registry.register(RedisHealth::new());
        registry.register(ExternalApiHealth::new());
    }
}
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
modules register themselves with the `HealthRegistry` at startup.

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
HealthModule::with_timeout(Duration::from_secs(3))
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
    use ironic::health::HealthRegistry;

    #[tokio::test]
    async fn test_database_health_ok() {
        let pool = PgPool::connect("postgres://localhost/testdb").await.unwrap();
        let indicator = DatabaseHealth { pool };
        let status = indicator.check().await;
        assert!(matches!(status, HealthStatus::Ok));
    }

    #[tokio::test]
    async fn test_health_endpoint_aggregation() {
        let mut registry = HealthRegistry::new(Duration::from_secs(5));
        registry.register(MockIndicator { name: "db", status: HealthStatus::Ok });
        registry.register(MockIndicator { name: "redis", status: HealthStatus::Ok });

        let response = registry.check_all().await;
        assert_eq!(response.status, "ok");
    }

    #[tokio::test]
    async fn test_health_endpoint_degraded_when_one_fails() {
        let mut registry = HealthRegistry::new(Duration::from_secs(5));
        registry.register(MockIndicator { name: "db", status: HealthStatus::Ok });
        registry.register(MockIndicator {
            name: "redis",
            status: HealthStatus::Unhealthy { error: "connection refused".into() },
        });

        let response = registry.check_all().await;
        assert_eq!(response.status, "unhealthy");
    }
}

struct MockIndicator {
    name: &'static str,
    status: HealthStatus,
}

impl HealthIndicator for MockIndicator {
    fn name(&self) -> &str { self.name }
    async fn check(&self) -> HealthStatus { self.status.clone() }
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

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Indicator blocks the async runtime | Avoid `.unwrap()` or blocking I/O inside `check()` |
| Forgetting to register indicators | Call `registry.register()` in `configure()` |
| Health check is slow | Keep `check()` under 1 second. Use timeouts for external calls |
| Not importing `HealthModule` | Add `HealthModule` to the module's `imports` |

## What you learned

- [x] `HealthModule` provides a built-in `GET /health` endpoint
- [x] `HealthIndicator` trait defines `name()` and `check()` for custom checks
- [x] Composite health returns `200` / `207` / `503` based on aggregate status
- [x] IntegrationHealth wiring auto-registers database/Redis indicators
- [x] Configurable timeout prevents slow checks from blocking the endpoint
- [x] Docker HEALTHCHECK integrates directly with the `/health` endpoint
