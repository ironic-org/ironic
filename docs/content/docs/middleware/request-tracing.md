---
title: Request Tracing
description: Automatic request IDs and tracing spans for every HTTP request — correlate logs, track requests across services.
---

# Request Tracing

## What you'll learn

- How `RequestTracing` adds correlation IDs and tracing spans to every request
- How to access the request ID in your handlers
- How to propagate request IDs to downstream services

`RequestTracing` is automatically registered as global middleware by the framework. It requires no feature flag and no manual setup.

---

## What it does

For every incoming request, `RequestTracing`:

1. **Generates or propagates a request ID.** If the client sent an `x-request-id` header, that value is used. Otherwise, a unique ID is generated (`rf-{timestamp}-{sequence}`).
2. **Inserts the ID into the request context** as a `RequestId` extension.
3. **Creates a tracing span** (`ironic.http.request`) with the method, URI, and response status code.
4. **Injects the ID into the response** as the `x-request-id` header.

```rust
// No setup needed — RequestTracing::new() is auto-registered
```

---

## Accessing the request ID

Read the `RequestId` from the request context in any middleware, interceptor, or handler:

```rust
use ironic::prelude::*;

fn handler(
    context: RequestContext,
) -> impl IntoFrameworkResponse {
    if let Some(request_id) = context.extension::<RequestId>() {
        tracing::info!(%request_id, "processing request");
    }
    "ok"
}
```

The `RequestId` type also implements `Display` and `Clone`:

```rust
let id: RequestId = context.extension::<RequestId>().unwrap().clone();
println!("{}", id); // rf-00000001782a3c40-0000000000000001
```

---

## Use cases

### Log correlation

Every event emitted inside a `RequestTracing` span automatically carries the request ID. When combined with [`RequestLogging`](./request-logging) or the [`TimeSeriesLayer`](../observability/logging), you can correlate access logs with application logs by request ID.

### Distributed tracing

The `x-request-id` header can be forwarded to downstream services, enabling end-to-end request tracking across a microservice architecture. For full OpenTelemetry integration (W3C trace context, OTLP export), see [Tracing & Telemetry](../observability/tracing).

### Client-side debugging

Returning `x-request-id` to clients lets them include the ID in support tickets, making it straightforward to look up the exact request in your logs.

---

## What you learned

- [x] `RequestTracing` is auto-registered globally — no setup required
- [x] Adds or propagates `x-request-id`, creates a tracing span
- [x] Access the ID via `context.extension::<RequestId>()`
- [x] The `x-request-id` header is returned in the response
