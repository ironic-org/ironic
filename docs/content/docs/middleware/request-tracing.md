---
title: Request Tracing
description: Auto-registered — adds request IDs and tracing spans to every request.
---

# Request Tracing

`RequestTracing` runs on every request automatically. No setup or feature flag needed.

## What it does

- Generates a unique `x-request-id` (or propagates one from the client)
- Creates a `tracing` span with method, URI, and status code
- Returns `x-request-id` in the response header

## How to use

```rust
use ironic::prelude::*;

// No registration needed — it's auto-registered

// Access the request ID in your handler
fn handler(context: RequestContext) -> impl IntoFrameworkResponse {
    if let Some(id) = context.extension::<RequestId>() {
        tracing::info!(%id, "processing request");
        format!("request_id: {id}")
    } else {
        "ok".to_string()
    }
}
```

## Per-controller or per-route

Use `#[middleware]` to apply `RequestTracing` to specific controllers or routes (e.g. when you opt out globally but want it on certain paths):

```rust
#[controller("/api")]
#[middleware(RequestTracing::new())]
pub struct ApiController;

#[get("/sensitive")]
#[middleware(RequestTracing::new())]
async fn sensitive(&self) -> Result<Json<()>, HttpError> {
    // tracing only on this route
}
```

## Propagation

Forward the `x-request-id` header to downstream services for end-to-end correlation:

```rust
let request_id = context
    .extension::<RequestId>()
    .map(|id| id.as_str().to_owned())
    .unwrap_or_default();

client
    .get("https://internal-api/data")
    .header("x-request-id", &request_id)
    .send()
    .await?;
```
