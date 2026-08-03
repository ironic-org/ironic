---
title: Provider Health
description: Monitor DI container health — track provider construction success/failure rates and surface them in /health endpoints.
---

# Provider Health

## What is it?

`Container::health()` returns per-provider construction statistics. Track which providers are failing, how often, and what the last error was — without guessing.

Every time the container resolves a provider it records the outcome in an internal registry. The registry is `Mutex`-guarded and cheap to read, so you can poll it from a health check or expose it in an operational endpoint.

> **Why this matters:** a provider that fails intermittently (e.g. a database pool that can't connect, a third-party SDK with flaky credentials) can be invisible in request logs. Provider health surfaces these construction failures explicitly.

## How to use

```rust
let health = container.health();

println!("{} providers registered", health.total_providers);

for (key, stats) in &health.providers {
    if stats.error_count > 0 {
        tracing::warn!(
            provider = ?key,
            constructs = stats.construct_count,
            errors = stats.error_count,
            last_error = stats.last_error,
            "provider has errors"
        );
    }
}
```

## ProviderHealth structure

```rust
/// Per-provider health statistics.
pub struct ProviderHealth {
    /// Total successful constructions.
    pub construct_count: u64,
    /// Total failed constructions.
    pub error_count: u64,
    /// Last error message, if any.
    pub last_error: Option<String>,
}

/// Consolidated health summary for the container.
pub struct ProviderHealthSummary {
    /// Total registered providers.
    pub total_providers: usize,
    /// Per-provider health data.
    pub providers: HashMap<ProviderKey, ProviderHealth>,
}
```

## Integration with /health

`Container::health()` is a synchronous snapshot — it pairs naturally with the framework's health indicators. Implement `HealthIndicator` and read the container stats from your readiness check:

```rust
use std::pin::Pin;
use std::future::Future;
use ironic::{HealthIndicator, HealthStatus};

struct ProviderHealthCheck {
    container: ironic::Container,
}

impl HealthIndicator for ProviderHealthCheck {
    fn name(&self) -> &str {
        "providers"
    }

    fn check_readiness(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        let health = self.container.health();
        let failing = health.providers.values()
            .filter(|h| h.error_count > 0)
            .count();
        let status = if failing > 0 {
            HealthStatus::Degraded {
                message: Some(format!("{failing} providers have construction errors")),
            }
        } else {
            HealthStatus::Ok
        };
        Box::pin(std::future::ready(status))
    }
}
```

The `HealthIndicator` trait provides two probes:

- **Liveness** (`check_liveness`) — is the process alive? Defaults to `Ok`.
- **Readiness** (`check_readiness`) — is the app ready to serve traffic? Use this for dependency-aware checks like provider health.

The old `check()` method is deprecated since v0.5.0; implement `check_readiness()` for new code.

> **See also:** the full `HealthIndicator` contract, composite aggregation, and endpoint wiring live in [Health Checks](/docs/observability/health-checks).

## What you learned

- [x] `Container::health()` exposes per-provider statistics
- [x] Track `construct_count`, `error_count`, `last_error`
- [x] Implement `HealthIndicator::check_readiness()` to surface failing providers in `/health/ready`
- [x] Distinguish liveness (process alive) from readiness (dependencies healthy)
