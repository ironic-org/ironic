---
title: Overview
description: Production monitoring — Prometheus metrics, distributed tracing, and structured logging for your Ironic application.
---

# Observability

## What you'll learn

- Expose Prometheus metrics at `/metrics`
- Add retry and circuit breaker for resilience
- Set up distributed tracing with OpenTelemetry
- Monitor your app in production

---

## Metrics (Prometheus)

Enable in `Cargo.toml`:

```toml
ironic = { features = ["metrics"] }
```

Add to your app:

```rust
use ironic::metrics::{MetricsConfig, MetricsLayer, MetricsModule};
use ironic::{AxumAdapter, FrameworkApplication};

#[derive(Module)]
#[module(imports = [MetricsModule])]      // ← Exposes GET /metrics
struct AppModule;

#[ironic::main]
async fn main() {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new().configure_router(|r| {
            r.layer(MetricsLayer::new(MetricsConfig::default()));
        }))
        .build().await.unwrap()
        .listen("127.0.0.1:3000").await.unwrap();
}
```

Visit `http://localhost:3000/metrics`:

```
# HELP ironic_http_requests_total Total HTTP requests
ironic_http_requests_total 1547
# HELP ironic_http_request_duration_seconds Request latency
ironic_http_request_duration_seconds_bucket{le="0.005"} 120
ironic_http_request_duration_seconds_bucket{le="0.1"} 890
```

### Prometheus scrape config

```yaml
scrape_configs:
  - job_name: 'my-api'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

## Resilience (Retry & Circuit Breaker)

```toml
ironic = { features = ["resilience"] }
```

### Retry with backoff

```rust
use ironic::resilience::{RetryConfig, RetryLayer};

AxumAdapter::new().configure_router(|r| {
    r.layer(RetryLayer::new(RetryConfig {
        max_retries: 3,
        base_delay_ms: 100,         // Start at 100ms
        backoff_multiplier: 2.0,    // Double each time
        max_delay_ms: 10_000,       // Cap at 10 seconds
        ..RetryConfig::default()
    }));
});
```

### Circuit breaker

```rust
use ironic::resilience::{CircuitBreakerConfig, CircuitBreakerLayer};
use std::time::Duration;

AxumAdapter::new().configure_router(|r| {
    r.layer(CircuitBreakerLayer::new(CircuitBreakerConfig {
        failure_threshold: 5,                    // Open after 5 failures
        recovery_timeout: Duration::from_secs(30), // Try again after 30s
        ..CircuitBreakerConfig::default()
    }));
});
```

Circuit breaker states:

```
Closed ──(5 failures)──► Open ──(30s timeout)──► Half-Open ──(2 successes)──► Closed
   ▲                                                                              │
   └──────────────────────────────────────────────────────────────────────────────┘
```

## Distributed Tracing

```toml
ironic = { features = ["telemetry"] }
```

```rust
use ironic::telemetry::{TelemetryConfig, init_tracing};

let _guard = init_tracing(TelemetryConfig {
    service_name: "my-api".into(),
    otlp_endpoint: Some("http://localhost:4317".into()),  // Jaeger/Tempo
    sample_rate: 1.0,
    ..TelemetryConfig::default()
});
```

Control log level with `RUST_LOG`:

```bash
RUST_LOG=info cargo run                # Normal
RUST_LOG=my_api=debug cargo run        # Your code at debug
RUST_LOG=my_api=trace,ironic=info cargo run  # Detailed traces
```

## What you learned

- [x] `MetricsLayer` records request metrics automatically
- [x] `GET /metrics` exposes Prometheus-compatible data
- [x] `RetryLayer` adds exponential backoff
- [x] `CircuitBreakerLayer` protects failing dependencies
- [x] `init_tracing()` enables distributed tracing
