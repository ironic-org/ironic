---
title: Observability
description: Production metrics, logging, and distributed tracing for the Ironic framework.
---

# Observability

Ironic provides built-in observability features for production monitoring.

## Metrics (Prometheus)

Enable the `metrics` feature in `Cargo.toml`:

```toml
ironic = { features = ["metrics"] }
```

### Quick start

Apply the `MetricsLayer` to your Axum router and import `MetricsModule`:

```rust
use ironic::prelude::*;
use ironic::metrics::{MetricsConfig, MetricsLayer, MetricsModule};

#[derive(Module)]
#[module(imports = [MetricsModule])]
struct AppModule;

#[ironic::main]
async fn main() {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(
            AxumAdapter::new()
                .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
        )
        .build().await.unwrap()
        .listen("127.0.0.1:3000").await.unwrap();
}
```

### Exposed metrics

| Metric | Type | Description |
|--------|------|-------------|
| `ironic_http_requests_total` | counter | Total number of HTTP requests |
| `ironic_http_responses_total` | counter | Responses by status code (`status="200"`, `"500"`, etc.) |
| `ironic_http_request_duration_seconds` | histogram | Request latency with pre-configured buckets |
| `ironic_http_requests_in_flight` | gauge | Currently executing requests |
| `ironic_info` | gauge | Framework version (`version="0.1.7"`) |

### Configuration

```rust
MetricsConfig {
    latency_buckets: vec![
        0.001, 0.005, 0.01, 0.025, 0.05,
        0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ],
    per_endpoint: true,  // set false to aggregate all paths
}
```

### Prometheus scrape config

```yaml
scrape_configs:
  - job_name: 'my-api'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

---

## Resilience (Retry & Circuit Breaker)

Enable the `resilience` feature:

```toml
ironic = { features = ["resilience"] }
```

### Retry with exponential backoff

```rust
use ironic::resilience::{RetryConfig, RetryLayer};

AxumAdapter::new().configure_router(|r| {
    r.layer(RetryLayer::new(RetryConfig {
        max_retries: 3,
        base_delay_ms: 100,
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        max_delay_ms: 10000,
        retryable_statuses: vec![408, 429, 500, 502, 503, 504],
    }));
});
```

### Circuit Breaker

```rust
use ironic::resilience::{CircuitBreakerConfig, CircuitBreakerLayer};
use std::time::Duration;

AxumAdapter::new().configure_router(|r| {
    r.layer(CircuitBreakerLayer::new(CircuitBreakerConfig {
        failure_threshold: 5,           // open after 5 failures
        success_threshold: 2,           // close after 2 successes in half-open
        recovery_timeout: Duration::from_secs(30),
        failure_statuses: vec![500, 502, 503, 504],
    }));
});
```

**State machine:** Closed → (threshold failures) → Open → (recovery timeout) → Half-Open → (threshold successes) → Closed

---

## Distributed Tracing (OpenTelemetry)

Enable the `telemetry` feature:

```toml
ironic = { features = ["telemetry"] }
```

### Local tracing

```rust
use ironic::telemetry::{TelemetryConfig, init_tracing};

#[ironic::main]
async fn main() {
    let _guard = init_tracing(TelemetryConfig::default());

    // All tracing spans are captured and logged
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build().await.unwrap()
        .listen("127.0.0.1:3000").await.unwrap();
}
```

### OTLP export to Jaeger/Tempo

```rust
let _guard = init_tracing(TelemetryConfig {
    service_name: "my-api".into(),
    otlp_endpoint: Some("http://localhost:4317".into()),
    sample_rate: 1.0,  // 100% sampling in development; lower in production
    ..TelemetryConfig::default()
});
```

Configure log level via `RUST_LOG` environment variable:
```bash
RUST_LOG=info cargo run
RUST_LOG=my_api=debug,ironic=trace cargo run
```
