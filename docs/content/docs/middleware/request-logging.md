---
title: Request Logging
description: Auto-registered — logs every request with method, URI, status, body sizes, and duration.
---

# Request Logging

`RequestLogging` runs on every request automatically. Enable the `logging` feature to persist events to `.logs/`.

## Enabling

```toml
ironic = { features = ["logging"] }
```

`logging` is included in default features — new projects have it out of the box.

## What it logs

Every request produces a structured tracing event under `ironic.http.access`:

| Field | Description |
|---|---|
| `http_method` | GET, POST, etc. |
| `http_uri` | Request path |
| `http_status_code` | 200, 404, 500, etc. |
| `http_request_body_size` | Request body in bytes |
| `http_response_body_size` | Response body in bytes |
| `http_duration_ms` | Wall-clock duration |
| `event_level` | `"info"` (2xx), `"warn"` (4xx), `"error"` (5xx/error) |
| `http_error_code` | Error code on handler failures |

With `TimeSeriesModule`, events are persisted to `.logs/YYYY-MM-DD.jsonl`:

```json
{"timestamp":"2026-07-17T10:30:00Z","level":"INFO","target":"ironic.http.access","fields":{"event_level":"info","http_method":"GET","http_uri":"/api/users","http_status_code":200,"http_duration_ms":12.34}}
```

## Per-controller or per-route

Apply `RequestLogging` to specific controllers or routes with `#[middleware]`:

```rust
#[controller("/api")]
#[middleware(RequestLogging::new())]
pub struct ApiController;

#[get("/sensitive")]
#[middleware(RequestLogging::new())]
async fn sensitive(&self) -> Result<Json<()>, HttpError> {
    // logging only on this route
}
```

## Opting out

```rust
FrameworkApplication::builder()
    .module(AppModule::definition())
    .platform(AxumAdapter::new())
    .without_request_logging()
    .build().await.unwrap();
```

## Manual registration

```rust
build_http_application_with_overrides(&graph, Vec::new())?
    .middleware(RequestLogging::new());
```
