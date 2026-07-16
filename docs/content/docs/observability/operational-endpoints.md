---
title: Operational Endpoints
description: Version endpoint, liveness probe, and readiness probe for production deployments.
---

# Operational Endpoints

## What you'll learn

- Use the `GET /version` endpoint to inspect build metadata
- Configure Kubernetes liveness and readiness probes
- Understand the liveness vs. readiness distinction
- Integrate operational endpoints with Docker HEALTHCHECK

---

## Overview

Ironic provides three operational endpoints for production deployments, all
registered automatically when you import `HealthModule`:

| Endpoint | Purpose | HTTP Status |
|----------|---------|-------------|
| `GET /health/live` | Liveness probe | Always `200` |
| `GET /health/ready` | Readiness probe | `200` or `503` |
| `GET /version` | Build metadata | `200` |

These endpoints are lightweight and have no side effects — they do not modify
any state or trigger background work.

---

## Version endpoint

The `GET /version` endpoint returns compile-time build metadata:

```json
{
  "git_sha": "a1b2c3d",
  "build_timestamp": "1719876543",
  "rust_version": "rustc 1.85.0 (1234abcd 2025-02-17)",
  "features": ["auth", "logging", "metrics", "openapi"],
  "version": "0.4.8"
}
```

### Fields

| Field | Description | Source |
|-------|-------------|--------|
| `git_sha` | Git commit SHA (short) | `GIT_SHA` env var or `git rev-parse --short HEAD` |
| `build_timestamp` | Build timestamp | `BUILD_TIMESTAMP` env var or Unix epoch |
| `rust_version` | Rust compiler version | `RUSTC_VERSION` env var or `rustc --version` |
| `features` | Active Cargo feature flags | Compiled with `cfg!(feature = "...")` |
| `version` | Crate version from `Cargo.toml` | `env!("CARGO_PKG_VERSION")` |

### Build reproducibility

The `build.rs` script captures these values at compile time. In CI, set the
`GIT_SHA` and `BUILD_TIMESTAMP` environment variables for accurate metadata.
Locally, the script falls back to running `git rev-parse --short HEAD` and the
current system time. If git is unavailable, values default to `"unknown"`.

---

## Liveness probe

The `GET /health/live` endpoint returns `200 OK` with:

```json
{"status": "alive"}
```

This endpoint does **not** execute any dependency health checks. It only
verifies that the HTTP server is running and capable of responding.

### Kubernetes configuration

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 15
  timeoutSeconds: 3
  failureThreshold: 3
```

### Docker HEALTHCHECK

```dockerfile
HEALTHCHECK --interval=15s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:3000/health/live || exit 1
```

---

## Readiness probe

The `GET /health/ready` endpoint aggregates all registered
[`HealthIndicator::check_readiness()`](./health-checks#healthindicator-trait)
results and returns the composite status:

```json
{
  "status": "ok",
  "checks": {
    "database": {"status": "ok"},
    "redis": {"status": "ok"}
  }
}
```

When all dependencies are healthy, the response is `200 OK`. If any dependency
is degraded or unhealthy, the response is `503 Service Unavailable`.

### Kubernetes configuration

```yaml
readinessProbe:
  httpGet:
    path: /health/ready
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 2
```

---

## Best practices

1. **Use `/health/live` for liveness, `/health/ready` for readiness** —
   Kubernetes treats these probes differently. Liveness restarts the pod;
   readiness removes it from service endpoints.

2. **Keep probes lightweight** — Liveness should return immediately. Readiness
   should have a short timeout (default 5 seconds per check).

3. **Customise check_readiness for each component** — Override
   `check_readiness()` on your `HealthIndicator` implementations when a
   component needs dependency-aware health logic. The default delegates to
   the deprecated `check()` method.

4. **Use the `/version` endpoint in CI/CD** — Include build metadata in
   deployment notifications and rollback decisions.

---

## What you learned

- [x] `GET /version` returns compile-time build metadata
- [x] `GET /health/live` is a lightweight process-alive check
- [x] `GET /health/ready` aggregates dependency readiness
- [x] Kubernetes probes integrate directly with the operational endpoints
- [x] Liveness probes restart pods; readiness probes control traffic routing
