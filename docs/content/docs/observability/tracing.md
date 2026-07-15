---
title: Tracing
description: Distributed tracing with OpenTelemetry — request IDs, custom spans, and log correlation.
---

# Tracing

## What you'll learn

- Enable automatic request tracing with `x-request-id` headers
- Export traces to Jaeger or Tempo via OTLP
- Correlate logs with request IDs
- Add custom spans and events to your handlers

---

## RequestTracing

Every incoming request receives a unique `x-request-id` and a tracing span named `ironic.http.request`:

```toml
ironic = { features = ["telemetry"] }
```

```rust
use ironic::telemetry::init_tracing;

let _guard = init_tracing(TelemetryConfig {
    service_name: "my-api".into(),
    ..TelemetryConfig::default()
});
```

With no additional configuration, each request gets:

- A `x-request-id` response header (UUID v4)
- A span `ironic.http.request` containing `http.method`, `http.url`, `http.status_code`
- Log lines automatically decorated with `request_id` and `span_id`

## Distributed tracing with OTLP

Export traces to an OTLP-compatible collector (Jaeger, Tempo, Honeycomb):

```rust
use ironic::telemetry::{TelemetryConfig, init_tracing};
use std::time::Duration;

let _guard = init_tracing(TelemetryConfig {
    service_name: "my-api".into(),
    otlp_endpoint: Some("http://localhost:4317".into()),
    sample_rate: 1.0,                    // 100% of traces
    batch_export_interval: Duration::from_secs(5),
    ..TelemetryConfig::default()
});
```

| Field | Default | Description |
|-------|---------|-------------|
| `service_name` | `"ironic-app"` | Identifies your service in trace UIs |
| `otlp_endpoint` | `None` | OTLP gRPC collector URL |
| `sample_rate` | `1.0` | Fraction of traces to export (0.0–1.0) |
| `batch_export_interval` | `5s` | How often to flush spans to the collector |

## Log correlation

When tracing is enabled, all `tracing` log events include span context:

```rust
use tracing::{info, error};

#[ironic::get("/users/:id")]
async fn get_user(id: Path<u64>) -> Result<Json<User>, AppError> {
    info!(user_id = *id, "Fetching user");

    let user = query_user(*id).await?;

    info!(user_id = *id, "User fetched successfully");
    Ok(Json(user))
}
```

Log output (structured JSON):

```json
{
  "timestamp": "2025-07-15T10:30:00Z",
  "level": "INFO",
  "message": "Fetching user",
  "user_id": 42,
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "span_id": "0123456789abcdef",
  "trace_id": "fedcba9876543210fedcba9876543210"
}
```

## Custom spans and events

Create child spans for expensive operations:

```rust
use tracing::{info_span, Instrument};

#[ironic::get("/search")]
async fn search(query: Query<SearchParams>) -> Result<Json<Vec<Item>>, AppError> {
    let search_span = info_span!("search", q = %query.q);

    async {
        // This block runs inside the "search" span
        let items = fetch_from_db(&query.q).await;
        info!(count = items.len(), "Search results");
        Ok(Json(items))
    }
    .instrument(search_span)
    .await
}
```

## Trace sampling

Control which traces are exported:

```rust
let config = TelemetryConfig {
    sample_rate: 0.1,  // Export 10% of all traces
    // Always sample errors:
    sample_on_error: true,
    ..TelemetryConfig::default()
};
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Missing `telemetry` feature flag | Add `ironic = { features = ["telemetry"] }` |
| No OTLP collector running | Start Jaeger locally: `docker run -p 4317:4317 jaegertracing/all-in-one` |
| All traces sampled at 1.0 in prod | Set `sample_rate: 0.1` or lower to avoid overwhelming the collector |
| Guard dropped too early | Hold the `_guard` for the lifetime of the application — dropping it tears down tracing |

## What you learned

- [x] `RequestTracing` auto-generates `x-request-id` headers and `ironic.http.request` spans
- [x] OTLP export sends traces to Jaeger, Tempo, or any OTLP collector
- [x] Logs are automatically correlated with request IDs and span IDs
- [x] Custom spans wrap expensive operations for detailed trace waterfalls
- [x] `sample_rate` and `sample_on_error` control trace volume
