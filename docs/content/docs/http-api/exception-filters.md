---
title: Exception Filters
description: Catch errors before they reach the client — transform raw errors into clean, structured JSON responses.
---

# Exception Filters

## What is an Exception Filter?

When your handler returns an `Err(HttpError::not_found(...))`, the framework needs to decide what to send back to the client. Without an exception filter, it sends a generic 500 or a raw error message. With an exception filter, you control exactly what the client sees.

**Simple analogy:** A restaurant kitchen makes mistakes — wrong order, burnt food, missing ingredient. The waiter (exception filter) catches these problems before they reach the customer's table. Instead of the customer seeing raw kitchen errors, the waiter says "I'm sorry, we're out of the salmon — would you like the cod instead?" with a polite, structured response.

```
Handler returns Err("POST_NOT_FOUND")
            │
            ▼
    ┌──────────────┐
    │ NotFoundFilter │  ← catches 404s
    └──────┬───────┘
            ▼
    {
      "error": "POST_NOT_FOUND",
      "message": "Blog post not found",
      "status": 404
    }
```

Without the filter, the client might get a raw 500 or an unstructured error string. With the filter, every error is a consistent JSON shape with proper HTTP status codes.

## Built-in errors

Ironic provides `HttpError` for common HTTP status codes. Use these in your services and controllers:

```rust
use ironic::prelude::*;

HttpError::not_found("USER_NOT_FOUND", "User #42 does not exist")   // → 404
HttpError::bad_request("INVALID_INPUT", "Email is required")        // → 400
HttpError::unauthorized("UNAUTHORIZED", "Invalid token")            // → 401
HttpError::forbidden("FORBIDDEN", "Admin access required")          // → 403
HttpError::internal("DB_ERROR", "Database connection failed")       // → 500
```

Each takes two strings: an **error code** (machine-readable, like `POST_NOT_FOUND`) and a **message** (human-readable, like "Blog post not found").

## Creating a custom filter

An exception filter implements the `ExceptionFilter` trait with one method — `catch()`:

```rust
use std::sync::Arc;
use ironic::{ExceptionFilter, FilterContext, FrameworkResponse, HttpError, HttpStatus};

pub struct NotFoundFilter;

impl ExceptionFilter for NotFoundFilter {
    fn catch(
        &self,
        error: &HttpError,
        _ctx: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError> {
        if error.status() == HttpStatus::NOT_FOUND {
            let body = ironic::json::json!({
                "error": error.code(),
                "message": error.message(),
                "status": 404,
            });
            Ok(FrameworkResponse::json(HttpStatus::NOT_FOUND, &body)?)
        } else {
            Err(error.clone())   // ← Pass through errors we don't handle
        }
    }
}
```

Key points:
- The `catch` method receives the error and a `FilterContext` (which has route metadata).
- If you handle the error, return `Ok(response)`.
- If you don't handle it, return `Err(error.clone())` — the next filter in the chain gets a chance.
- Filters run in registration order. The first one to return `Ok` wins.

## Route-level — `.exception_filter()` builder

Apply to a single `RouteDefinition` using the dot notation:

```rust
use std::sync::Arc;
use ironic::{HttpMethod, RouteDefinition, handler_fn};

let route = RouteDefinition::new(
    HttpMethod::GET,
    "/users/:id",
    "show",
    handler_fn(|_controller: Arc<MyService>, _arguments| {
        async move {
            Err::<FrameworkResponse, HttpError>(HttpError::not_found(
                "POST_NOT_FOUND",
                "Post 42 was not found",
            ))
        }
    }),
)
.expect("route path must be valid")
.exception_filter(Arc::new(NotFoundFilter));
```

The filter runs specifically for this route. If the handler returns a 404, `NotFoundFilter::catch()` transforms it into a clean JSON response. If another error type occurs, the `Err(error.clone())` passes it through to the next filter or the global handler.

## Global — GlobalErrorMiddleware

For catching ALL errors across the entire application, use a middleware wrapper instead of an exception filter. Register it as the outermost middleware in your `main.rs`:

```rust
use ironic::{Middleware, MiddlewareNext, PipelineFuture, RequestContext};

struct GlobalErrorMiddleware;

impl Middleware for GlobalErrorMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            match next.run(context).await {
                Ok(response) => Ok(response),
                Err(error) => {
                    let body = ironic::json::json!({
                        "error": error.code(),
                        "message": error.message(),
                        "status": error.status().as_u16(),
                    });
                    ironic::FrameworkResponse::json(error.status(), &body)
                }
            }
        })
    }
}

// In main():
let application = FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(GlobalErrorMiddleware)  // ← outermost, catches everything
    .middleware(SecurityHeadersMiddleware::new(...))
    .build().await.unwrap();
```

This ensures NO unhandled error reaches the client as a raw trace. Every error gets transformed into `{"error", "message", "status"}` JSON.

**Place it first** in the middleware chain so it wraps the entire application — security headers, rate limits, CORS, and all routes are inside its try/catch.

## Route-level vs Global

| Approach | Scope | Use for |
|---|---|---|
| `.exception_filter()` on `RouteDefinition` | One route | Specific error handling (404 → custom response) |
| `GlobalErrorMiddleware` | Every route | Catch-all safety net (all errors → JSON) |
| Both together | Full coverage | Route-specific catches first, global catches the rest |

Typical setup: `GlobalErrorMiddleware` as the outermost middleware (safety net), plus route-specific `.exception_filter()` for custom error responses on critical endpoints.

## Filter priority

More specific filters run first:

```
Route filter → Controller filter → Global filter → Default handler
 (tried first)    (tried next)        (last resort)    (returns 500)
```

The pipeline tries each filter in order. The first filter to return `Ok(response)` wins — no further filters run. If all filters pass through with `Err(error)`, the default handler returns a generic 500.

## When to use Exception Filters vs Guards

| You want to... | Use |
|---|---|
| Prevent a request from reaching the handler (auth check) | `#[guard]` |
| Transform an error into a nice response | `.exception_filter(Arc::new(...))` |
| Catch ALL unhandled errors globally | `GlobalErrorMiddleware` |
| Run code after the handler regardless of error | `#[interceptor]` |
| Return a custom error message from the handler | `HttpError::not_found(...)` directly |

**Rule:** Guards say "no" before the handler runs. Exception filters clean up after the handler fails.

## Common mistakes

| Mistake | Fix |
|---|---|
| Returning `Err(...)` for auth failures from a guard, expecting the exception filter to catch it | Guards return `Deny` to short-circuit — exception filters only catch handler errors |
| Catching all errors without passing through unknowns | Return `Err(error.clone())` for errors you don't handle |
| Filter returning `Ok` response with wrong status code | Match the status in your response to the original error's status |
| Expecting access to extracted parameters in the filter | The handler hasn't run yet (or has failed) — use `FilterContext` for metadata |
| Forgetting `Arc::new(...)` | Exception filters must be wrapped in `Arc` |

## What you learned

- [x] Exception filters catch errors and transform them into clean responses
- [x] Implement `ExceptionFilter::catch()` to handle specific error statuses
- [x] Use `.exception_filter(Arc::new(...))` on `RouteDefinition` for route-level filtering
- [x] Use `GlobalErrorMiddleware` for a global catch-all safety net
- [x] Filters chain — first to `Ok` wins, pass through with `Err` to try the next
- [x] Use guards for access control, exception filters for error transformation
