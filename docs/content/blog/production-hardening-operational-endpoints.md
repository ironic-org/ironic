---
title: "Production-hardening 2: Health probes, version endpoint, and build metadata"
description: "Liveness and readiness probe endpoints for Kubernetes (GET /health/live, GET /health/ready). Build metadata endpoint (GET /version) with git SHA, build timestamp, Rust version, and active features. HealthIndicator trait split for liveness vs readiness distinction."
date: "Jul 16, 2026"
author: "Ironic Team"
---

# Production-hardening 2: Health probes, version endpoint, and build metadata

Kubernetes needs liveness and readiness probes to manage pod lifecycle. Operators need build metadata to trace deployments. This post covers Ironic's operational endpoints and how they integrate with container orchestration.

---

## Operational endpoints

Importing `HealthModule` registers four endpoints:

```rust
#[derive(Module)]
#[module(imports = [HealthModule])]
struct AppModule;
```

| Endpoint | Purpose | HTTP Status |
|----------|---------|-------------|
| `GET /health` | Composite health (backward compatible) | 200 / 207 / 503 |
| `GET /health/live` | Liveness probe — process alive? | Always 200 |
| `GET /health/ready` | Readiness probe — dependencies healthy? | 200 / 503 |
| `GET /version` | Build metadata | 200 |

### Liveness probe

`GET /health/live` returns `{"status": "alive"}` with HTTP 200. It executes **no** dependency checks — it only verifies the HTTP server is running.

```yaml
# Kubernetes livenessProbe
livenessProbe:
  httpGet:
    path: /health/live
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 15
```

### Readiness probe

`GET /health/ready` aggregates all registered `HealthIndicator::check_readiness()` results:

```json
{
  "status": "ok",
  "checks": {
    "database": {"status": "ok"},
    "redis": {"status": "ok"}
  }
}
```

Returns 200 when all dependencies are healthy, 503 when any dependency is degraded or unhealthy.

```yaml
# Kubernetes readinessProbe
readinessProbe:
  httpGet:
    path: /health/ready
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
```

### Version endpoint

`GET /version` returns compile-time build metadata:

```json
{
  "git_sha": "a1b2c3d",
  "build_timestamp": "1719876543",
  "rust_version": "rustc 1.85.0 (1234abcd 2025-02-17)",
  "features": ["auth", "logging", "metrics", "openapi"],
  "version": "0.4.8"
}
```

The `build.rs` script captures these values at compile time. In CI, set `GIT_SHA` and `BUILD_TIMESTAMP` environment variables for accurate metadata. Locally, it falls back to `git rev-parse --short HEAD` and the current system time.

---

## Liveness vs. readiness

The `HealthIndicator` trait now distinguishes two probe types:

```rust
pub trait HealthIndicator: Send + Sync {
    fn name(&self) -> &str;

    #[deprecated(since = "0.5.0", note = "use `check_readiness` instead")]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>>;

    /// Default: always returns Ok (process is alive)
    fn check_liveness(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(std::future::ready(HealthStatus::Ok))
    }

    /// Default: delegates to deprecated `check()` for backward compatibility
    fn check_readiness(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        #[allow(deprecated)]
        self.check()
    }
}
```

- **Liveness** (`check_liveness`): Is the process itself alive? Override only for fatal internal state detection. Defaults to `Ok`.
- **Readiness** (`check_readiness`): Is the component ready to serve traffic? Override for dependency-aware logic. Defaults to calling the old `check()` for backward compatibility.

The old `check()` method is deprecated but not removed. Existing `HealthIndicator` implementations continue to work unchanged — `check_readiness()` delegates to `check()` by default.

---

## What this means for production

Before this change, production deployments had to build their own version endpoint and health probes from scratch. Now:

- **Kubernetes-native probes**: `/health/live` and `/health/ready` work directly with `livenessProbe` and `readinessProbe` configuration
- **Deployment traceability**: `/version` gives operators git SHA, build timestamp, Rust version, and active feature flags
- **Backward compatible**: The existing `/health` endpoint and `HealthIndicator::check()` continue to work unchanged
- **Zero-config**: Import `HealthModule` and all endpoints are registered automatically
