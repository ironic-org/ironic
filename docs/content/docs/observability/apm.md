---
title: Application Performance Monitoring (APM)
description: End-to-end APM setup for Ironic — OpenTelemetry traces, Prometheus metrics, error rates, health probes, and vendor integrations.
---

# Application Performance Monitoring (APM)

APM answers three questions in production: **how slow is it**, **how often does
it fail**, and **where is the time going**. Ironic gives you the four signals
out of the box — traces, metrics, logs, and health — through standard
OpenTelemetry and Prometheus protocols, so you can plug into any vendor instead
of being locked into one.

```
┌────────────────────────────────────────────────────────────┐
│                     Your Ironic service                     │
│                                                            │
│  traces (OTLP gRPC :4317)      metrics (GET /metrics)      │
│  logs (structured, trace_id)   health (/health/live ready) │
└──────────┬──────────────────────────┬──────────────────────┘
           │ OTLP                     │ Prometheus scrape
           ▼                          ▼
   ┌───────────────┐          ┌─────────────┐
   │ APM collector │          │ Prometheus  │
   │ Jaeger/Tempo/ │          └──────┬──────┘
   │ Honeycomb/    │                 ▼
   │ Datadog OTel  │          ┌─────────────┐
   └───────────────┘          │  Grafana    │
                              └─────────────┘
```

---

## What you'll learn

- Enable traces, metrics, and health in one `Cargo.toml`
- Export traces to Jaeger, Grafana Tempo, or any OTLP-compatible APM
- Scrape latency, throughput, and error-rate metrics with Prometheus
- Wire health probes for your orchestrator
- Map the pipeline to Datadog, New Relic, Honeycomb, and SigNoz
- Alert on the RED metrics (Rate, Errors, Duration)

---

## 1. Enable the features

```toml
# Cargo.toml
ironic = { version = "1", features = ["telemetry", "metrics", "logging"] }
```

| Feature | Provides |
|---------|----------|
| `telemetry` | OpenTelemetry traces, OTLP export, W3C `traceparent` propagation |
| `metrics` | Prometheus `/metrics` endpoint, request latency/rate/error metrics |
| `logging` | Structured JSON logs correlated with `trace_id` / `span_id` |

---

## 2. Traces — where the time goes

Initialise tracing once, at startup, and **hold the guard for the application's
lifetime** (dropping it tears down and flushes the exporter).

```rust
use ironic::telemetry::{init_tracing, TelemetryConfig};
use std::time::Duration;

fn main() {
    let _guard = init_tracing(TelemetryConfig {
        service_name: "orders-service".into(),
        otlp_endpoint: Some("http://localhost:4317".into()), // APM collector
        sample_rate: 0.1,                                    // 10% of requests
        batch_interval: Duration::from_secs(2),              // flush every 2s
        propagate_context: true,                             // W3C traceparent
    });

    // ... your Application::builder() / .listen().await
}
```

### What you get automatically

- Every HTTP request becomes an `ironic.http.request` span with `http.method`,
  `http.url`, and `http.status_code`
- Handler logs carry `trace_id` + `span_id`, so logs and traces cross-reference
- Outgoing requests propagate `traceparent`, joining multi-service traces into
  one waterfall

### `TelemetryConfig`

| Field | Default | Description |
|-------|---------|-------------|
| `service_name` | crate name | Appears as `service.name` in trace UIs |
| `otlp_endpoint` | `None` | OTLP gRPC collector URL; `None` = local-only traces |
| `batch_interval` | `5s` | How often spans are flushed to the collector |
| `sample_rate` | `1.0` | Fraction of traces exported (see [Sampling](#sampling)) |
| `propagate_context` | `true` | Inject W3C `traceparent` on outgoing requests |

### Sampling

| `sample_rate` | Sampler | Suggested use |
|---------------|---------|---------------|
| `1.0` | `AlwaysOn` | Development / low traffic |
| `0.1`–`0.5` | `TraceIdRatioBased` | Staging, most production services |
| `0.01`–`0.05` | `TraceIdRatioBased` | High-throughput services |
| `0.0` | `AlwaysOff` | Disable export |

---

## 3. Metrics — latency, rate, and errors

The `metrics` feature registers a Prometheus scrape endpoint and measures every
request automatically.

```rust
use ironic::metrics::{MetricsConfig, MetricsLayer, MetricsModule};

#[derive(Module)]
#[module(imports = [MetricsModule])]   // ← registers GET /metrics
struct AppModule;

AxumAdapter::new().configure_router(|r| {
    r.layer(MetricsLayer::new(MetricsConfig::default()));
});
```

Scrape `http://localhost:3000/metrics`. The APM-relevant metrics are:

| Metric | Type | What it powers |
|--------|------|----------------|
| `ironic_http_requests_total` | Counter | Request rate (RPS) |
| `ironic_http_request_duration_seconds` | Histogram | Latency percentiles (p50/p95/p99) |
| `ironic_http_requests_in_flight` | Gauge | Concurrency |
| `ironic_http_errors_total` | Counter | Error rate (5xx) |
| `ironic_http_endpoint_status_total` | Counter | Per-endpoint 2xx/4xx/5xx |

### The RED method

APM dashboards are built from three numbers. All three come from the metrics above:

```text
# Rate — requests per second
rate(ironic_http_requests_total[5m])

# Errors — 5xx per second
rate(ironic_http_errors_total[5m])

# Duration — p99 latency
histogram_quantile(0.99, sum by (le) (rate(ironic_http_request_duration_seconds_bucket[5m])))
```

---

## 4. Health probes — so your orchestrator knows

Import `HealthModule` to expose liveness, readiness, and version endpoints:

```rust
use ironic::prelude::*;

#[derive(Module)]
#[module(imports = [HealthModule])]
struct AppModule;
```

| Endpoint | Purpose | Status |
|----------|---------|--------|
| `GET /health/live` | Process is up | Always `200` |
| `GET /health/ready` | Dependencies healthy | `200` or `503` |
| `GET /version` | Build metadata | `200` |

```yaml
# kubernetes.yaml
livenessProbe:
  httpGet: { path: /health/live,  port: 3000 }
readinessProbe:
  httpGet: { path: /health/ready, port: 3000 }
```

---

## 5. Vendor integrations

Ironic exports **standard OTLP** (traces) and **Prometheus** (metrics). Point
the pipeline at any vendor — the code never changes, only `otlp_endpoint` and
the scrape target.

| Vendor | Traces | Metrics | Notes |
|--------|--------|---------|-------|
| **Grafana stack** | OTLP → Tempo | Prometheus → scrape | Correlate logs in Loki by `trace_id` |
| **Jaeger** | OTLP gRPC :4317 | — | Great for local dev |
| **Datadog** | OTLP → Datadog intake | Agent scrapes `/metrics` | Or run the OTel→Datadog exporter |
| **New Relic** | OTLP gRPC | New Relic Prometheus remote write | |
| **Honeycomb** | OTLP gRPC | — | Strong high-cardinality analysis |
| **SigNoz** | OTLP gRPC | OTLP metrics | Self-hosted, OTel-native |

### Quick start: Grafana Tempo + Prometheus (all self-hosted)

```yaml
# docker-compose.yml
services:
  tempo:
    image: grafana/tempo:latest
    command: ["-config.file=/etc/tempo-config.yml"]
    ports: ["4317:4317"]        # OTLP gRPC for traces
  prometheus:
    image: prom/prometheus:latest
    ports: ["9090:9090"]
    volumes: ["./prometheus.yml:/etc/prometheus/prometheus.yml"]
  grafana:
    image: grafana/grafana:latest
    ports: ["3001:3000"]
```

Point `otlp_endpoint` at `http://localhost:4317` and add a Prometheus scrape job:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: orders-service
    metrics_path: /metrics
    static_configs:
      - targets: ['localhost:3000']
```

### Quick start: Datadog

```bash
DD_OTLP_CONFIG_RECEIVER_PROTOCOLS_GRPC_ENDPOINT=0.0.0.0:4317 \
DD_API_KEY=your-key docker run --rm -p 4317:4317 \
  gcr.io/datadoghq/agent:latest
```

Then set `otlp_endpoint: Some("http://localhost:4317")`. The agent converts OTLP
spans to Datadog APM traces. For metrics, point the Datadog agent at
`/metrics` with the Prometheus check.

---

## 6. Logs — the correlation glue

Enable `logging` and use structured fields; every log line automatically carries
`request_id`, `span_id`, and `trace_id` when tracing is active:

```rust
#[get("/users/:id")]
async fn get_user(id: Path<u64>) -> Result<Json<User>, AppError> {
    tracing::info!(user_id = *id, "Fetching user");  // carries trace context
    // ...
}
```

In your log aggregator, filter by `trace_id` to see the full request log —
then jump from the log to the trace waterfall in your APM UI.

---

## 7. Alerts

Base alerts on the RED metrics. Prometheus alert rules:

```text
groups:
  - name: apm
    rules:
      - alert: HighErrorRate
        expr: rate(ironic_http_errors_total[5m]) > 1
        for: 2m
        annotations:
          summary: "Error rate above 1/sec for 2 minutes"

      - alert: HighLatency
        expr: |
          histogram_quantile(0.99,
            sum by (le) (rate(ironic_http_request_duration_seconds_bucket[5m]))) > 1
        for: 5m
        annotations:
          summary: "p99 latency above 1 second for 5 minutes"

      - alert: EndpointErrorRate
        expr: rate(ironic_http_endpoint_status_total{status="5xx"}[5m]) > 0.1
        annotations:
          summary: "Endpoint {{ $labels.endpoint }} has elevated 5xx rate"
```

---

## Production checklist

- [ ] `telemetry` + `metrics` + `logging` features enabled
- [ ] `init_tracing` guard held for the application lifetime
- [ ] `service_name` set per service (matches your deployment name)
- [ ] `sample_rate` at 0.1 or lower in production
- [ ] `MetricsLayer` + `MetricsModule` wired; `/metrics` reachable
- [ ] `HealthModule` imported; probes configured in K8s/Docker
- [ ] `otlp_endpoint` pointed at your APM collector
- [ ] Prometheus scraping `/metrics` with a stable job name
- [ ] Alerts on rate, errors, and duration
- [ ] Logs aggregated and correlated by `trace_id`

---

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Dropping the `_guard` early | Hold it for the whole process — `Drop` flushes pending spans |
| `sample_rate: 1.0` in high-traffic prod | Set 0.01–0.1 to avoid overwhelming the collector |
| No collector at `otlp_endpoint` | Start one, or set `None` (local-only) during development |
| Missing `MetricsLayer` | `MetricsModule` exposes the endpoint, but the layer records the data |
| High-cardinality route params | Set `per_endpoint: false` or normalize paths (see the metrics doc) |
| No `traceparent` on outbound calls | `propagate_context` defaults to true; call `inject_trace_context()` manually where needed |
| Health endpoint doing heavy work | Liveness must return immediately; keep readiness checks short |

---

## Related docs

- [Tracing & Telemetry](/docs/observability/tracing) — spans, custom spans, OTLP config
- [Metrics](/docs/observability/metrics) — buckets, custom counters/gauges/histograms
- [Error Counter Metric](/docs/observability/error-metrics) — `ironic_http_errors_total` alerts
- [Health Checks](/docs/observability/health-checks) — `HealthIndicator` trait
- [Operational Endpoints](/docs/observability/operational-endpoints) — probes, `/version`
- [Distributed Tracing](/docs/observability/distributed-tracing) — cross-service traces
