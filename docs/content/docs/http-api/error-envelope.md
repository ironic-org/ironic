---
title: Error Envelope
description: Production-grade error responses with request_id, timestamp, and structured details for log correlation.
---

# Error Envelope

## What is it?

When an API returns an error, the client needs enough information to debug the issue. A bare `{"code":"ERR"}` isn't enough. The error envelope adds `request_id` and `timestamp` to every error response — making it easy to correlate client errors with server logs.

## Why it matters

**Without the envelope:**
```json
{"status": 404, "code": "POST_NOT_FOUND", "message": "Blog post not found"}
```
You can't tell WHEN this happened or WHICH request it was. If a user reports an error, you have to guess which server log line matches.

**With the envelope:**
```json
{
  "status": 404,
  "code": "POST_NOT_FOUND",
  "message": "Blog post not found",
  "timestamp_ms": 1720100000000,
  "request_id": "req-a1b2c3d4"
}
```
Now you know the exact millisecond and the exact request ID. Search your logs for `req-a1b2c3d4` and see the full request lifecycle.

## How to use

```rust
use ironic::prelude::*;

fn handler(context: &mut RequestContext) -> Result<Json<User>, HttpError> {
    let request_id = context.extension::<RequestId>()
        .map(|id| id.as_str());

    Err(HttpError::not_found("USER_NOT_FOUND", "User not found"))
        .map_err(|e| {
            FrameworkResponse::error_with_tracing(
                IronStatus::NOT_FOUND,
                e.code(),
                e.message(),
                request_id,   // ← from RequestTracing middleware
            )
    })
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | `u16` | HTTP status code |
| `code` | `&str` | Machine-readable error code |
| `message` | `String` | Human-readable message |
| `timestamp_ms` | `u128` | Unix timestamp in milliseconds |
| `request_id` | `Option<&str>` | From `RequestTracing` middleware, for log correlation |

## Inline error transformation

The `ExceptionExt::exception()` method transforms errors inline without `match` blocks:

```rust
use ironic::ExceptionExt;

self.auth.login(&username, &password)
    .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
```

**Before:**
```rust
let tokens = match self.auth.login(&username, &password) {
    Ok(t) => t,
    Err(e) => return Err(HttpError::unauthorized("LOGIN_FAILED", e.message())),
};
```

**After:**
```rust
let tokens = self.auth.login(&username, &password)
    .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
```

## Filter-based inline catch

Route an error through an existing `ExceptionFilter`:

```rust
use std::sync::Arc;
use ironic::ExceptionExt;

self.repo.find(id)
    .exception_catch(Arc::new(NotFoundFilter))?;
```

## What you learned

- [x] `error_with_tracing()` adds `request_id` + `timestamp_ms` to every error
- [x] `.exception(|e| ...)` transforms errors inline without match blocks
- [x] `.exception_catch(Arc::new(filter))` routes through existing filters
- [x] Use `RequestTracing` middleware to set up `request_id` for the envelope
