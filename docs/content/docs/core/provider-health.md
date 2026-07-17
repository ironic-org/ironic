---
title: Provider Health
description: Monitor DI container health — track provider construction success/failure rates and expose in /health endpoints.
---

# Provider Health

## What is it?

`Container::health()` returns per-provider construction statistics. Track which providers are failing, how often, and what the last error was.

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
pub struct ProviderHealth {
    pub construct_count: u64,   // Successful constructions
    pub error_count: u64,       // Failed constructions
    pub last_error: Option<String>, // Last error message
}

pub struct ProviderHealthSummary {
    pub total_providers: usize,
    pub providers: HashMap<ProviderKey, ProviderHealth>,
}
```

## Integration with /health

Extend your health endpoint to report unhealthy providers:

```rust
impl HealthIndicator for ProviderHealthCheck {
    fn check_health(&self) -> HealthStatus {
        let health = self.container.health();
        let failing = health.providers.values()
            .filter(|h| h.error_count > 0)
            .count();
        if failing > 0 {
            HealthStatus::Degraded(format!("{failing} providers have errors"))
        } else {
            HealthStatus::Healthy
        }
    }
}
```

## What you learned

- [x] `Container::health()` exposes per-provider statistics
- [x] Track `construct_count`, `error_count`, `last_error`
- [x] Integrate with `HealthIndicator` for operational monitoring
