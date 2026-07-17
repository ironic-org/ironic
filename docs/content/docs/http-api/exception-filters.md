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
use ironic::{ExceptionFilter, FilterContext, FrameworkResponse, HttpError, HttpStatus};

pub struct NotFoundFilter;

impl ExceptionFilter for NotFoundFilter {
    fn catch(
        &self,
        error: &HttpError,
        _ctx: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError> {
        if error.status() == HttpStatus::NOT_FOUND {
            let body = serde_json::json!({
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

## Where to apply filters

### Controller-level — `#[exception]` attribute

Apply to all routes in a controller:

```rust
#[controller("/blogs")]
#[exception(NotFoundFilter)]
#[derive(Injectable)]
struct BlogsController;
```

Every route in `BlogsController` inherits the `NotFoundFilter`. If any handler returns a 404, the filter catches it and returns the structured JSON response.

### Route-level — `.exception_filter()` builder

Apply to a single route definition:

```rust
let route = RouteDefinition::new(HttpMethod::GET, "/users/:id", "show", handler_fn(show_user))?
    .exception_filter(Arc::new(NotFoundFilter));
```

### Global — catches everything

```rust
FrameworkApplication::builder()
    .module(AppModule::definition())
    .platform(AxumAdapter::new())
    .exception_filter(Arc::new(GlobalErrorFilter))
    .build().await.unwrap();
```

Global filters run last, after all route and controller filters. This is where you put catch-all handlers — convert `InternalServerError` into a safe public message, or log every unhandled error.

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
| Transform an error into a nice response | `#[exception]` |
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

## What you learned

- [x] Exception filters catch errors and transform them into clean responses
- [x] Implement `ExceptionFilter::catch()` to handle specific error statuses
- [x] Use `#[exception]` on controllers, `.exception_filter()` on routes, or globally
- [x] Filters chain — first to `Ok` wins, pass through with `Err` to try the next
- [x] Use guards for access control, exception filters for error transformation
