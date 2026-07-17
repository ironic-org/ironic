---
title: Request Logging
description: Structured access logging for every HTTP request — method, URI, status, body sizes, and duration — persisted to time-series storage.
---

# Request Logging

## What you'll learn

- How `RequestLogging` captures structured access logs for every request
- How it integrates with `TimeSeriesLayer` for time-series persistence
- How to opt out with `.without_request_logging()`
- The fields included in each log event

`RequestLogging` is automatically registered as global middleware by the framework. It emits structured `tracing` events that the `TimeSeriesLayer` (from the `logging` feature) captures and persists to `.logs/YYYY-MM-DD.jsonl`.

---

## What it does

For every request-response cycle, `RequestLogging` emits a single `tracing::info!` event under the `ironic.http.access` target with these fields:

| Field | Type | Description |
|---|---|---|
| `event_level` | `"info"` / `"warn"` / `"error"` | Severity based on status code |
| `http_method` | `&str` | HTTP method (GET, POST, ...) |
| `http_uri` | `&str` | Raw request URI |
| `http_status_code` | `u16` | Response status code |
| `http_request_body_size` | `u64` | Request body size in bytes |
| `http_response_body_size` | `u64` | Response body size in bytes |
| `http_duration_ms` | `f64` | Wall-clock duration in milliseconds |
| `http_error_code` | `&str` | (On error only) Error code string |

The `event_level` classification:

| Status range | `event_level` |
|---|---|
| 200–399 | `"info"` |
| 400–499 | `"warn"` |
| 500–599 | `"error"` |
| Handler error | `"error"` |

---

## Integration with time-series logging

When the `logging` feature is enabled (it is in the default feature set), the `TimeSeriesLayer` captures all tracing events and writes them to `.logs/YYYY-MM-DD.jsonl`. No additional configuration is needed:

```toml
# Already included by default
ironic = { features = ["logging"] }
```

Each access log is written as a JSON Line entry:

```json
{"timestamp":"2026-07-17T10:30:00Z","level":"INFO","target":"ironic.http.access","message":"","fields":{"event_level":"info","http_method":"GET","http_uri":"/api/users","http_status_code":200,"http_request_body_size":0,"http_response_body_size":256,"http_duration_ms":12.34}}
```

To learn more about the logging system, custom storage backends, and configuration, see [Structured Logging](../observability/logging).

---

## Opting out

If you do not want automatic request logging, disable it on the application builder:

```rust
use ironic::prelude::*;

let app = FrameworkApplication::builder()
    .module(AppModule::definition())
    .platform(AxumAdapter::new())
    .without_request_logging()   // ← disables default RequestLogging
    .build().await.unwrap();
```

You can also register it manually on a `CompiledHttpApplication` if you need fine-grained control:

```rust
use ironic::prelude::*;

let app = build_http_application(&graph)?
    .middleware(RequestLogging::new());
```

---

## Use cases

### Access log audit trail

Every request is automatically recorded with method, URI, status, and timing. This provides a complete access log for debugging, compliance, and analytics — without writing any logging code.

### Performance monitoring

The `http_duration_ms` field captures wall-clock duration for every request. Aggregate across status codes and routes to detect slowdowns:

```
# Example: find slow routes from your logs
jq 'select(.fields.http_duration_ms > 1000)' .logs/2026-07-17.jsonl
```

### Error tracking

Requests that result in handler errors include the `http_error_code` field, making it easy to filter and analyze failure modes:

```
# Find all errors with their codes
jq 'select(.fields.event_level == "error") | {uri: .fields.http_uri, code: .fields.http_error_code}' .logs/2026-07-17.jsonl
```

---

## What you learned

- [x] `RequestLogging` is auto-registered globally — no setup required
- [x] Logs method, URI, status, body sizes, duration per request
- [x] Integrates with `TimeSeriesLayer` for JSON Lines persistence
- [x] Use `.without_request_logging()` on the builder to opt out
- [x] Events are classified by severity: `info` / `warn` / `error` based on status
