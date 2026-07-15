---
title: Metrics
description: Expose Prometheus metrics, measure request latency, and track custom application metrics.
---

# Metrics

## What you'll learn

- Enable the `MetricsModule` to expose a `/metrics` endpoint
- Use `MetricsLayer` to measure request latency, count, and status codes
- Configure latency buckets and per-endpoint tracking
- Define custom metrics via `MetricsStore`

---

## Enabling metrics

Add the feature flag in `Cargo.toml`:

```toml
ironic = { features = ["metrics"] }
```

Import `MetricsModule` into your app module:

```rust
use ironic::metrics::MetricsModule;

#[derive(Module)]
#[module(imports = [MetricsModule])]
struct AppModule;
```

This registers a `GET /metrics` endpoint that returns Prometheus-formatted data.

## MetricsLayer

`MetricsLayer` is a tower layer that records every HTTP request automatically:

```rust
use ironic::metrics::{MetricsConfig, MetricsLayer};

AxumAdapter::new().configure_router(|r| {
    r.layer(MetricsLayer::new(MetricsConfig::default()));
});
```

### What gets measured

| Metric | Type | Labels |
|--------|------|--------|
| `ironic_http_requests_total` | Counter | `method`, `path`, `status` |
| `ironic_http_request_duration_seconds` | Histogram | `method`, `path`, `status` |
| `ironic_http_requests_in_flight` | Gauge | — |

## MetricsConfig

```rust
use ironic::metrics::MetricsConfig;

let config = MetricsConfig {
    latency_buckets: vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0],
    per_endpoint: true,   // Track metrics per route (e.g. /users vs /products)
    ..MetricsConfig::default()
};

AxumAdapter::new().configure_router(|r| {
    r.layer(MetricsLayer::new(config));
});
```

| Field | Default | Description |
|-------|---------|-------------|
| `latency_buckets` | `[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]` | Histogram buckets in seconds |
| `per_endpoint` | `true` | When `true`, adds a `path` label. Set to `false` for high-cardinality routes |

## Prometheus output format

```
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

## Custom metrics

Register application-specific metrics through `MetricsStore`:

```rust
use ironic::metrics::MetricsStore;
use ironic::Inject;

fn record_order_placed(metrics: Inject<MetricsStore>) {
    metrics
        .counter("orders_placed_total")
        .with_label("status", "confirmed")
        .inc();
}
```

Supported metric types:
- `counter(name)` — increment-only counter
- `gauge(name)` — value that goes up and down
- `histogram(name)` — distribution with configurable buckets

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Missing `metrics` feature flag | Add `ironic = { features = ["metrics"] }` to `Cargo.toml` |
| No `MetricsModule` import | Add `MetricsModule` to your module's `imports` array |
| Layer registered after routing | Register `MetricsLayer` before route handlers for accurate timing |
| High-cardinality route params | Set `per_endpoint: false` or bucket routes like `/users/:id` |

## What you learned

- [x] `MetricsModule` exposes a `GET /metrics` endpoint
- [x] `MetricsLayer` automatically measures request latency and count
- [x] `MetricsConfig` controls bucket sizes and per-endpoint tracking
- [x] `MetricsStore` lets you define custom counters, gauges, and histograms
- [x] Output is Prometheus-compatible for scraping by Grafana, VictoriaMetrics, etc.
