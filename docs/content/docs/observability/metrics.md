---
title: Metrics
description: Expose Prometheus metrics, measure request latency, and track custom application metrics.
---

# Metrics

## What you'll learn

- Enable the `MetricsModule` to expose a `/metrics` endpoint
- Use `MetricsLayer` to measure request latency, count, and status codes
- Configure latency buckets and per-endpoint tracking
- Define custom metrics via `MetricsRegistry`
- Understand histogram bucket mechanics
- How to test metrics in integration tests

---

## Enabling metrics

```toml
ironic = { features = ["metrics"] }
```

## Quick start

```rust
use ironic::metrics::MetricsModule;

#[derive(Module)]
#[module(imports = [MetricsModule])]
struct AppModule;
```

This registers a `GET /metrics` endpoint that returns Prometheus-formatted data.
Visit `http://localhost:3000/metrics` to see the output.

## MetricsLayer

`MetricsLayer` is a tower layer that records every HTTP request automatically:

```rust
use ironic::metrics::{MetricsConfig, MetricsLayer};

AxumAdapter::new().configure_router(|r| {
    r.layer(MetricsLayer::new(MetricsConfig::default()));
});
```

### What gets measured

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `ironic_http_requests_total` | Counter | `method`, `path`, `status` | Total request count |
| `ironic_http_request_duration_seconds` | Histogram | `method`, `path`, `status` | Latency distribution |
| `ironic_http_requests_in_flight` | Gauge | — | Concurrent requests |

## MetricsConfig

```rust
use ironic::metrics::MetricsConfig;

let config = MetricsConfig {
    latency_buckets: vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0],
    per_endpoint: true,
    ..MetricsConfig::default()
};

AxumAdapter::new().configure_router(|r| {
    r.layer(MetricsLayer::new(config));
});
```

| Field | Default | Description |
|-------|---------|-------------|
| `latency_buckets` | `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]` | Histogram bucket upper bounds in seconds. Must be sorted ascending. |
| `per_endpoint` | `true` | When `true`, adds a `path` label. Set to `false` for high-cardinality routes. |

### Choosing histogram buckets

The right buckets depend on your application's latency profile:

| Use case | Suggested buckets |
|----------|------------------|
| API (fast, <100ms) | `[0.001, 0.005, 0.01, 0.025, 0.05, 0.1]` |
| API (standard) | `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]` |
| File upload / heavy I/O | `[0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0]` |
| Background job worker | `[0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0]` |

### Latency histogram internals

Latency is recorded using a fixed-size `[AtomicU64; 13]` array — one counter per
bucket, with one extra slot for overflow.  This means:

- **Lock-free:** Recording a latency value is a single atomic increment
- **No allocation at request time:** Buckets are pre-allocated at config time
- **No sorting at scrape time:** Bucket counts are read directly, no histogram
  reconstruction needed
- **13 buckets max:** The array is fixed-size; if you configure more than 12
  buckets, the 13th slot holds overflow

```text
Bucket counters (AtomicU64 × 13):

  0.005  0.01  0.025  0.05  0.1  0.25  0.5  1.0  2.5  5.0  10.0  +Inf  overflow
  [ 42 ] [ 38 ] [ 25 ] [ 18 ] [ 12 ] [ 8 ] [ 5 ] [ 3 ] [ 2 ] [ 1 ] [ 0 ] [ 0 ] [ 0 ]
```

A request with 300ms latency increments the `0.25` bucket (the smallest bucket
whose upper bound is ≥ 300ms).

## Per-endpoint tracking

When `per_endpoint: true` (default), each route gets its own label set:

```
ironic_http_requests_total{method="GET",path="/users",status="200"} 1547
ironic_http_requests_total{method="POST",path="/users",status="201"} 342
ironic_http_requests_total{method="GET",path="/users/42",status="200"} 89
```

### Cardinality warning

When more than 1000 unique `(method, path)` combinations are tracked, a warning
is logged at `WARN` level.  This helps you catch accidental high-cardinality
from dynamic route parameters (e.g., `/users/{id}` where `{id}` varies widely).

To reduce cardinality:
1. Set `per_endpoint: false` to aggregate all paths into one label set
2. Normalize paths in middleware before they reach the metrics layer

## Prometheus output format

```text
# HELP ironic_http_requests_total Total HTTP requests
# TYPE ironic_http_requests_total counter
ironic_http_requests_total{method="GET",path="/users",status="200"} 1547
ironic_http_requests_total{method="POST",path="/users",status="201"} 342
# HELP ironic_http_request_duration_seconds Request latency
# TYPE ironic_http_request_duration_seconds histogram
ironic_http_request_duration_seconds_bucket{le="0.005"} 120
ironic_http_request_duration_seconds_bucket{le="0.01"} 310
ironic_http_request_duration_seconds_bucket{le="+Inf"} 1889
ironic_http_request_duration_seconds_sum 12.45
ironic_http_request_duration_seconds_count 1889
```

## Custom metrics with MetricsRegistry

### Counter — monotonically increasing

```rust
use ironic::metrics::MetricsRegistry;
use ironic::Inject;

fn record_order(registry: Inject<MetricsRegistry>) {
    let counter = registry.counter("orders_placed_total", "Total orders placed");

    // Basic increment
    counter.inc();

    // Increment by a custom amount
    counter.inc_by(5);
}
```

### Gauge — value that goes up and down

```rust
fn track_connections(registry: Inject<MetricsRegistry>) {
    let gauge = registry.gauge("active_connections", "Active connections");

    gauge.set(42);       // set absolute value
    gauge.inc();         // +1
    gauge.dec();         // -1
}
```

### Histogram — distribution with buckets

```rust
fn record_payment_latency(registry: Inject<MetricsRegistry>) {
    let histogram = registry.histogram(
        "payment_processing_seconds",
        "Payment processing latency",
    );

    let start = std::time::Instant::now();
    process_payment().await;
    let elapsed = start.elapsed();

    histogram.record(elapsed.as_secs_f64());
}
```

### Metrics registry API

```rust
impl MetricsRegistry {
    /// Creates or retrieves a counter.
    pub fn counter(&self, name: &str, help: &str) -> Arc<Counter>;

    /// Creates or retrieves a gauge.
    pub fn gauge(&self, name: &str, help: &str) -> Arc<Gauge>;

    /// Creates or retrieves a histogram with Prometheus default bucket boundaries.
    pub fn histogram(&self, name: &str, help: &str) -> Arc<Histogram>;
}

impl Counter {
    pub fn inc(&self);
    pub fn inc_by(&self, n: u64);
}

impl Gauge {
    pub fn set(&self, n: u64);
    pub fn inc(&self);
    pub fn dec(&self);
}

impl Histogram {
    pub fn record(&self, value: f64);
}
```

## Registering custom metrics

```rust
use std::sync::Arc;
use ironic::metrics::{MetricsModule, MetricsRegistry};
use ironic::Inject;

#[derive(Module)]
#[module(imports = [MetricsModule])]
struct AppModule;

// Custom module that registers metrics providers
#[derive(Module)]
#[module(providers = [MetricsRegistry])]
struct BillingModule;

impl BillingModule {
    #[provider]
    fn provide_billing_metrics(registry: Inject<MetricsRegistry>) -> BillingMetrics {
        BillingMetrics {
            orders: registry.counter("orders_placed_total", "Total orders placed"),
            revenue: registry.counter("revenue_total", "Total revenue"),
            latency: registry.histogram("payment_duration_seconds", "Payment duration in seconds"),
        }
    }
}

struct BillingMetrics {
    orders: Arc<Counter>,
    revenue: Arc<Counter>,
    latency: Arc<Histogram>,
}
```

## Testing metrics

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_metrics_endpoint_returns_prometheus() {
        let app = build_app();
        let response = app
            .oneshot(Request::get("/metrics").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("ironic_http_requests_total"));
    }

    #[tokio::test]
    async fn test_histogram_bucket_assignment() {
        let registry = MetricsRegistry;
        let histogram = registry.histogram("test", "Test histogram");

        histogram.record(0.3);
        histogram.record(0.7);
        histogram.record(1.5);
        histogram.record(3.0);

        // Verify via scrape output
        let output = scrape();
        assert!(output.contains("test_bucket"));
        assert!(output.contains("test_count"));
    }

    #[test]
    fn test_counter_operations() {
        let registry = MetricsRegistry;
        let counter = registry.counter("test_counter", "Test counter");
        assert_eq!(counter.value(), 0);

        counter.inc();
        assert_eq!(counter.value(), 1);

        counter.inc_by(5);
        assert_eq!(counter.value(), 6);
    }
}
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Missing `metrics` feature flag | Add `ironic = { features = ["metrics"] }` to `Cargo.toml` |
| No `MetricsModule` import | Add `MetricsModule` to your module's `imports` array |
| Layer registered after routing | Register `MetricsLayer` before route handlers for accurate timing |
| High-cardinality route params | Set `per_endpoint: false` or normalize paths before the metrics layer |
| Unsorted bucket bounds | Buckets must be sorted ascending — the histogram panics on unsorted input |
| Counter reset on restart | Metrics are in-memory. Use Prometheus `rate()` to handle resets gracefully |

## What you learned

- [x] `MetricsModule` exposes a `GET /metrics` Prometheus endpoint
- [x] `MetricsLayer` automatically records latency, request count, and in-flight gauge
- [x] `MetricsConfig` controls bucket sizes and per-endpoint tracking
- [x] `MetricsRegistry` lets you define custom counters, gauges, and histograms
- [x] Histogram buckets are `[AtomicU64; 13]` — lock-free, no allocation, no scrape-time sorting
- [x] Cardinality warnings fire at >1000 unique `(method, path)` label combinations
- [x] Custom metrics are injectable through the DI container as singleton providers
