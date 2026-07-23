---
title: Tracing & Telemetry
description: Distributed tracing with OpenTelemetry — request IDs, OTLP export, W3C trace propagation, custom spans, and log correlation.
---

# Tracing & Telemetry

## What you'll learn

- Enable automatic request tracing with `x-request-id` headers
- Export traces to Jaeger or Tempo via OTLP with gRPC
- W3C trace context propagation for multi-service traces
- Configure sampling rates (AlwaysOn, AlwaysOff, TraceIdRatioBased)
- Correlate logs with request IDs
- Add custom spans, events, and attributes
- Test tracing in integration tests

---

## Telemetry feature

```toml
ironic = { features = ["telemetry"] }
```

The `telemetry` feature enables:
- `tracing-subscriber` — structured async-aware logging
- `opentelemetry` — trace API
- `opentelemetry-otlp` — gRPC export to collector (Jaeger, Tempo, Honeycomb)
- `opentelemetry_sdk` — span processor, batching, sampling
- `tracing-opentelemetry` — bridges `tracing` spans to OpenTelemetry

## Quick start

```rust
use ironic::telemetry::{TelemetryConfig, init_tracing};

let _guard = init_tracing(TelemetryConfig {
    service_name: "my-api".into(),
    ..TelemetryConfig::default()
});
```

Without an OTLP endpoint, spans are created locally but not exported — the
logs still carry trace IDs for local debugging.  This is the recommended setup
for development.

## TelemetryConfig

```rust
use std::time::Duration;

let config = TelemetryConfig {
    service_name: "payment-service".into(),
    otlp_endpoint: Some("http://localhost:4317".into()),
    sample_rate: 0.25,  // export 25% of traces
    batch_export_interval: Duration::from_secs(2),  // flush every 2s
};
```

| Field | Default | Description |
|-------|---------|-------------|
| `service_name` | `"ironic-app"` | Identifies your service in trace UIs (Jaeger, Tempo, etc.) |
| `otlp_endpoint` | `None` | OTLP gRPC collector URL. Set to `Some(...)` to enable export |
| `sample_rate` | `1.0` | Fraction of traces to export (0.0–1.0) |
| `batch_export_interval` | `5s` | How often to flush span batches to the collector |

When `otlp_endpoint` is `None` (the default):
- Spans are still created and can be inspected programmatically
- Log events carry `trace_id` and `span_id` for local correlation
- No network overhead from trace export

## Production setup guide

### 1. Start Jaeger locally

```bash
docker run -d --name jaeger \
  -p 4317:4317 \
  -p 16686:16686 \
  jaegertracing/all-in-one:latest
```

### 2. Configure your app

```bash
# .env or environment
OTLP_ENDPOINT=http://localhost:4317
SAMPLE_RATE=0.1
```

### 3. Load config

```rust
let otlp = std::env::var("OTLP_ENDPOINT").ok();
let sample_rate: f64 = std::env::var("SAMPLE_RATE")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(1.0);

let _guard = init_tracing(TelemetryConfig {
    service_name: "my-api".into(),
    otlp_endpoint: otlp.map(|e| e.into()),
    sample_rate,
    ..TelemetryConfig::default()
});
```

### 4. View traces

Open http://localhost:16686 (Jaeger UI) to see trace waterfalls.

## RequestTracing

With the `telemetry` feature, every incoming request automatically receives:

```rust
// No manual wiring — RequestTracing is auto-registered.
// Each request creates a span named "ironic.http.request" with:
//   - http.method      → "GET", "POST", etc.
//   - http.url         → "/users/42"
//   - http.status_code → 200, 404, etc.

// Your handler logs are automatically correlated:
#[get("/users/:id")]
async fn get_user(id: Path<u64>) -> Result<Json<User>, AppError> {
    tracing::info!(user_id = *id, "Fetching user");  // ← carries trace_id + span_id
    // ...
}
```

## W3C trace context propagation

Ironic injects the `traceparent` header for outgoing HTTP requests, enabling
distributed trace correlation across service boundaries:

```
traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
```

### Format breakdown

| Part | Value |
|------|-------|
| Version | `00` |
| Trace ID | `0af7651916cd43dd8448eb211c80319c` (16 bytes, hex) |
| Span ID | `b7ad6b7169203331` (8 bytes, hex) |
| Trace flags | `01` (sampled) |

### Manual propagation example

When calling an external HTTP API from a handler, use the `inject_trace_context()` helper:

```rust
use ironic::telemetry::inject_trace_context;

async fn call_external_api() -> Result<(), HttpError> {
    let client = reqwest::Client::new();

    // Build a request and inject trace context into it
    let mut request = http::Request::builder()
        .uri("https://api.example.com/data")
        .body(())
        .unwrap();

    inject_trace_context(&mut request);

    let resp = client
        .get("https://api.example.com/data")
        .headers(request.headers().clone())
        .send()
        .await?;

    Ok(())
}
```

## Sampling

```rust
// Export 10 % of all traces
let config = TelemetryConfig {
    sample_rate: 0.1,
    ..TelemetryConfig::default()
};
```

| `sample_rate` | Sampler |
|---|---|
| `1.0` | `AlwaysOn` — export every trace |
| `0.0` | `AlwaysOff` — export nothing |
| Any other value | `TraceIdRatioBased` — probabilistic sampling |

### Sampling strategy by environment

| Environment | Suggested `sample_rate` | Rationale |
|---|---|---|
| Development | `1.0` | See all traces, debugging |
| Staging | `0.5` | Balance of visibility and cost |
| Production (low traffic) | `0.1` | 10% is statistically significant |
| Production (high traffic) | `0.01`–`0.05` | 1–5% for high-throughput services |

## Log correlation

When tracing is enabled, all `tracing` log events automatically include span
context.  Here is what a structured log entry looks like:

```json
{
  "timestamp": "2025-07-15T10:30:00Z",
  "level": "INFO",
  "target": "my_api::handlers",
  "message": "User fetched successfully",
  "user_id": 42,
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "span_id": "0123456789abcdef",
  "trace_id": "fedcba9876543210fedcba9876543210"
}
```

You can use these fields to:
- Filter logs by `trace_id` in your log aggregator (e.g., Loki, Elasticsearch)
- Jump from a log line to the corresponding trace waterfall in Jaeger
- Correlate logs across microservices by `trace_id`

## Custom spans

### Method 1: `info_span!` + `.instrument()`

```rust
use tracing::Instrument;

#[get("/search")]
async fn search(query: Query<SearchParams>) -> Result<Json<Vec<Item>>, AppError> {
    let span = tracing::info_span!("search", q = %query.q);

    async {
        let items = fetch_from_db(&query.q).await;
        tracing::info!(count = items.len(), "Search completed");
        Ok(Json(items))
    }
    .instrument(span)
    .await
}
```

### Method 2: `in_span()`

```rust
use tracing::Span;

fn process_payment(amount: f64) -> Result<(), PaymentError> {
    Span::current().record("amount", amount);

    tracing::info_span!("process_payment", provider = %"stripe")
        .in_scope(|| {
            // This code runs inside the "process_payment" span
            stripe_charge(amount)
        })
}
```

### Method 3: Span events

```rust
use tracing::info_span;

let span = info_span!("cache_lookup", cache_hit = false);
let _guard = span.enter();

// Add events (structured log messages on the span timeline)
tracing::event!(
    tracing::Level::INFO,
    cache_hit = false,
    latency_ms = 12,
    "Cache miss, fetching from origin"
);

// Update span attributes after the fact
span.record("cache_hit", true);
```

## Testing tracing

```rust
#[cfg(test)]
mod tests {
    use tracing_subscriber::fmt::TestWriter;

    fn init_test_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("test=debug")
            .with_writer(TestWriter::new)
            .try_init();
    }

    #[tokio::test]
    async fn test_span_attributes() {
        init_test_tracing();

        let span = tracing::info_span!("test_span", user_id = 42);
        async {
            // Verify tracing context exists
            assert!(tracing::Span::current().is_some());
        }
        .instrument(span)
        .await;
    }
}
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Missing `telemetry` feature flag | Add `ironic = { features = ["telemetry"] }` to `Cargo.toml` |
| No OTLP collector running | Start Jaeger: `docker run -p 4317:4317 jaegertracing/all-in-one` |
| All traces sampled at 1.0 in prod | Set `sample_rate: 0.1` or lower to avoid overwhelming the collector |
| Guard dropped too early | Hold the `_guard` for the lifetime of the application — dropping it tears down tracing |
| `traceparent` not propagated | Use `inject_trace_context()` before making outgoing HTTP calls |
| Forgetting to flush on shutdown | The guard's `Drop` implementation flushes — ensure it lives until the process exits |

## What you learned

- [x] `RequestTracing` auto-generates `x-request-id` headers and `ironic.http.request` spans
- [x] OTLP export sends traces to Jaeger, Tempo, or any OTLP collector via gRPC
- [x] `inject_trace_context()` injects W3C `traceparent` for multi-service propagation
- [x] `sample_rate` maps to `AlwaysOn` / `AlwaysOff` / `TraceIdRatioBased` samplers
- [x] Logs are automatically correlated with `request_id`, `span_id`, and `trace_id`
- [x] Custom spans wrap expensive operations for detailed trace waterfalls
- [x] Span events and attributes enrich traces with structured metadata
- [x] The `_guard` must live for the application's lifetime
