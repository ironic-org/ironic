---
title: Health Checks
description: Built-in health endpoint, custom health indicators, and Docker HEALTHCHECK integration.
---

# Health Checks

## What you'll learn

- Use the built-in `GET /health` endpoint
- Add custom health indicators for databases, Redis, and external services
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

## Health check aggregation

The overall status is **up** only when all indicators report healthy. If any indicator is down, the endpoint returns `503`:

| Indicators | Overall |
|------------|---------|
| All up | `200 { status: "ok" }` |
| One or more down | `503 { status: "degraded" }` |
| Unknown | `503 { status: "unknown" }` |

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
- [x] Custom `HealthIndicator` implementations check databases, Redis, and external services
- [x] Aggregated status is `up` only when all components are healthy
- [x] Docker HEALTHCHECK integrates directly with the `/health` endpoint
