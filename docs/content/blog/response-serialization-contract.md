---
title: "Response and IntoResponse — the protocol-neutral boundary"
description: "How handler return values become responses through a simple trait, why the structured error format stays the same regardless of the HTTP server, and where the adapter boundary actually lives."
date: "2026-07-15"
author: "Ironic Team"
---

# Response and IntoResponse — the protocol-neutral boundary

Every HTTP framework has a response type. Most of them tie it directly to the underlying server runtime — Actix Web returns `HttpResponse`, Axum returns `Response<Body>`. Ironic refuses to do that. Instead, it defines its own protocol-neutral response type and a single trait that converts any handler return value into it. The adapter layer then translates that into whatever the platform needs.

---

## The response type: nothing but status, headers, body

Open `crates/ironic-http/src/response.rs:29` and you'll find exactly three fields:

```rust
pub struct Response {
    status: HttpStatus,
    headers: HeaderMap,
    body: Body,
}
```

No streams, no channels, no async wiring. `Body` is just two variants: `Empty` or `Bytes(Vec<u8>)`. This is deliberate. The framework never streams — it materializes the entire response before handing it across the adapter boundary. If you need streaming, build it on the adapter side, not here.

`HttpStatus` is a re-export of `http::StatusCode`. `HeaderMap` is a re-export of `http::HeaderMap`. These are the only dependencies on the `http` crate in the entire response module — and they are the same types every Rust HTTP library already understands.

---

## The conversion trait: one method, infinite return types

The trait at `response.rs:131` is small enough to fit in a screenshot:

```rust
pub trait IntoResponse {
    fn into_framework_response(self) -> Result<Response, HttpError>;
}
```

That's the entire contract. Any type that implements this trait can be returned from a handler. The framework provides six implementations out of the box:

- `Response` itself — passthrough identity.
- `()` — `204 No Content`.
- `String` and `&'static str` — `200 OK` with the text as a byte body.
- `Json<T>` — `200 OK` with `application/json` content-type.
- `Result<T, E>` — unwraps `Ok` or `Err`, converting each arm separately.

The `Result` impl is the one that matters most in practice. A handler returns `Result<Json<User>, HttpError>`. The `Ok` arm takes the JSON path: `Serialize` into bytes, set the content-type header, wrap in `Response`. The `Err` arm hits `HttpError`'s own implementation.

---

## Structured errors: a contract across every adapter

When `HttpError` converts to a response (`error.rs:87`), it calls `Response::error()`:

```rust
impl IntoResponse for HttpError {
    fn into_framework_response(self) -> Result<Response, HttpError> {
        Ok(Response::error(self.status, self.code, self.message))
    }
}
```

The `error()` method serializes a JSON body with three keys: `status`, `code`, and `message`. So every error response from Ironic looks like this:

```json
{
  "status": 403,
  "code": "RF_HTTP_GUARD_DENIED",
  "message": "Access to this route was denied"
}
```

The `code` field uses a stable, namespaced format (`RF_HTTP_*`). These codes are the framework's public error API. Clients can switch on them. Middleware can inspect them. Tooling can generate documentation from them.

---

## The adapter never knows about Ironic error codes

This is where the architecture pays off. The Axum adapter has exactly two functions for converting responses (`ironic-platform-axum/src/lib.rs:332`):

```rust
fn framework_response(response: Response) -> Response {
    let (status, headers, body) = response.into_parts();
    // map Body::Empty -> Body::empty()
    // map Body::Bytes -> Body::from(bytes)
    // copy status and headers
}

fn error_response(error: HttpError) -> Response {
    match error.into_framework_response() {
        Ok(response) => framework_response(response),
        Err(_) => framework_response(Response::empty(INTERNAL_SERVER_ERROR)),
    }
}
```

The adapter never inspects the error code. It never reads the JSON body. It never knows what `RF_HTTP_GUARD_DENIED` means. It just calls `into_framework_response()` on the `HttpError`, gets back a `Response`, and mechanically translates headers and bytes. If tomorrow Ironic adds fifty new error codes, the Axum adapter is unchanged. If someone writes an Actix adapter next week, they write the same two mechanical functions.

The boundary is clean:

```
Handler result
    │
    ▼
IntoResponse::into_framework_response()
    │
    ▼
Response (status + headers + bytes)
    │
    ▼
Platform adapter (mechanical translation)
    │
    ▼
Axum Response / Actix HttpResponse / Hyper Response
```

The framework never produces platform-specific types. The adapter never knows framework semantics. They only meet at `Response` — three fields, fully materialized, no secrets.
