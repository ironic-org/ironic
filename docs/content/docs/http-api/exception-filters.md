---
title: Exception Filters
description: Handle errors gracefully — catch exceptions, transform them into user-friendly responses, and log what went wrong.
---

# Exception Filters

## What you'll learn

- Catch errors at the controller, route, or application level
- Transform raw errors into structured JSON responses
- Create custom error codes for your API
- Return proper HTTP status codes (404, 400, 500)

## The big picture

Errors happen. Exception filters let you handle them **gracefully** instead of crashing:

```
Handler returns Err(...)
      │
      ▼
┌──────────────────┐
│ Exception Filter │ ← Transforms the error
└──────┬───────────┘
       ▼
  {
    "error": "USER_NOT_FOUND",
    "message": "User with id 42 was not found",
    "status": 404
  }
```

## Built-in errors

Ironic provides `HttpError` with common HTTP status codes:

```rust
use ironic::prelude::*;

// In your service or controller:
HttpError::not_found("USER_NOT_FOUND", "User #42 does not exist")
// → 404 with error code

HttpError::bad_request("INVALID_INPUT", "Email is required")
// → 400 with error code

HttpError::unauthorized("UNAUTHORIZED", "Invalid token")
// → 401 with error code

HttpError::forbidden("FORBIDDEN", "Admin access required")
// → 403 with error code

HttpError::internal("DB_ERROR", "Database connection failed")
// → 500 with error code
```

## Creating a custom filter

Catch specific errors and transform them:

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
            Ok(FrameworkResponse::error(
                HttpStatus::NOT_FOUND,
                "CUSTOM_NOT_FOUND",
                format!("Resource not found: {}", error.message()),
            ))
        } else {
            Err(error.clone())  // ← Pass through errors we don't handle
        }
    }
}
```

## Where to apply filters

### Route level (most specific)

```rust
RouteDefinition::new(HttpMethod::GET, "/:id", "get_user", handler)
    .unwrap()
    .exception_filter(Arc::new(NotFoundFilter))
```

### Controller level

```rust
ControllerDefinition::new::<UserController>("/users", provider)
    .unwrap()
    .exception_filter(Arc::new(NotFoundFilter))
```

### Application level (catches everything)

```rust
FrameworkApplication::builder()
    .exception_filter(Arc::new(GlobalErrorFilter))
    .build().await.unwrap();
```

## Filter priority

More specific filters run first. If they pass, the next level runs:

```
Route filter ──► Controller filter ──► Global filter ──► Default handler
  (tried first)     (tried next)         (last resort)     (returns 500)
```

> **Best practice:** Put specific filters at the route level and a general "catch-all" at the global level.

## Try it yourself

1. Create a `ValidationErrorFilter` that catches validation errors
2. Transform them into `{"error": "VALIDATION", "fields": {...}}`
3. Apply it at the controller level
4. Send invalid data and verify the response format

## What you learned

- [x] Use `HttpError` for common HTTP errors (404, 400, 401, 403, 500)
- [x] Create custom filters with `ExceptionFilter` trait
- [x] Apply filters at route, controller, or global level
- [x] Filters cascade from most specific to most general
