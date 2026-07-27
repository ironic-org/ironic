---
title: Request Logging
description: Auto-registered — logs every request with method, URI, status, body sizes, and duration.
---

# Request Logging

`RequestLogging` is **auto-added by `Application::build()`** — you don't need to add it manually. Every request produces a structured tracing event under `ironic.http.access`.

Enable the `logging` feature (default in new projects) to persist events to `.logs/`:

```toml
ironic = { features = ["logging"] }
```

## What it logs

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

## Custom Logger (override the default)

Disable the built-in logger and add your own:

```rust
use std::time::Instant;
use ironic::prelude::*;

pub struct JsonLogger;

impl Middleware for JsonLogger {
    fn handle<'a>(
        &'a self,
        ctx: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let start = Instant::now();
            let method = ctx.request().method().to_string();
            let uri = ctx.request().uri().to_string();

            let result = next.run(ctx).await;

            let duration = start.elapsed();
            let status = match &result {
                Ok(r) => r.status().as_u16(),
                Err(e) => e.status().as_u16(),
            };

            tracing::info!(
                target: "custom.access",
                method = %method,
                path = %uri,
                status,
                dur_ms = duration.as_secs_f64() * 1000.0,
            );

            result
        })
    }
}

// Wire it up:
Application::builder()
    .module(AppModule::definition())
    .without_request_logging()
    .middleware(JsonLogger)
    .platform(AxumAdapter::new())
    .build()
    .await
    .expect("application must initialise");
```

## Disabling (without replacement)

```rust
Application::builder()
    .module(AppModule::definition())
    .platform(AxumAdapter::new())
    .without_request_logging()
    .build().await.unwrap();
```
