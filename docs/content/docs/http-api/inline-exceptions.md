---
title: Inline Exception Handling
description: Handle errors inline on Result values with .exception() closures and .exception_catch() filters — no boilerplate match statements.
---

# Inline Exception Handling

The `ExceptionExt` trait extends `Result<T, HttpError>` with two methods for inline error handling — no `match`, no `map_err` boilerplate.

## `.exception()` — closure-based

Transform any error with a simple closure:

```rust
use ironic::ExceptionExt;

self.auth
    .login(&dto.username, &dto.password)
    .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
```

The closure receives the `HttpError` and returns a new one. This is equivalent to `.map_err(|e| ...)` but reads as a fluent chain.

**Common patterns:**

```rust
// Remap error codes
find_user(id)
    .exception(|e| HttpError::not_found("USER_NOT_FOUND", "no such user"))?;

// Preserve the original message
validate(input)
    .exception(|e| HttpError::bad_request("VALIDATION", e.message()))?;

// Attach context to internal errors
db_query()
    .exception(|e| HttpError::internal("DB_QUERY", format!("query failed: {}", e.message())))?;
```

## `.exception_catch()` — filter-based

Pass the error through an `ExceptionFilter`:

```rust
use std::sync::Arc;
use ironic::ExceptionExt;

self.auth
    .login(&dto.username, &dto.password)
    .exception_catch(Arc::new(NotFoundFilter))?;
```

If the filter returns `Ok(response)`, the error is replaced with the response body text. If the filter returns `Err(error)`, the original error passes through unchanged.

This is useful when you have a reusable filter that you want to apply inline to a single operation, rather than registering it on a route or globally.

## When to use which

| Situation | Use |
|---|---|
| Remap one error code to another | `.exception(\|e\| ... )` |
| Add context or metadata to an error | `.exception(\|e\| ... )` |
| Apply a reusable ExceptionFilter inline | `.exception_catch(Arc::new(...))` |
| Catch ALL errors on a route pipeline | `.exception_filter(Arc::new(...))` on RouteDefinition |
| Catch ALL errors globally | `GlobalExceptionMiddleware` as outermost middleware |

## How it works

```rust
pub trait ExceptionExt<T> {
    fn exception<F>(self, f: F) -> Result<T, HttpError>
    where F: FnOnce(HttpError) -> HttpError;

    fn exception_catch(self, filter: Arc<dyn ExceptionFilter>) -> Result<T, HttpError>;
}

impl<T> ExceptionExt<T> for Result<T, HttpError> {
    fn exception<F>(self, f: F) -> Result<T, HttpError>
    where F: FnOnce(HttpError) -> HttpError {
        self.map_err(f)
    }

    fn exception_catch(self, filter: Arc<dyn ExceptionFilter>) -> Result<T, HttpError> {
        self.or_else(|error| {
            let ctx = FilterContext::new(RouteMetadata::default());
            match filter.catch(&error, &ctx) {
                Ok(response) => {
                    let body = String::from_utf8_lossy(response.body().as_bytes());
                    Err(HttpError::internal(error.code(), body.to_string()))
                }
                Err(e) => Err(e),
            }
        })
    }
}
```

## Example — blog-api login

From the blog-api example, the login endpoint uses `.exception()` to remap auth errors:

```rust
#[post("/login")]
async fn login(&self, #[body] dto: LoginDto) -> Result<Json<TokenResponse>, HttpError> {
    let tokens = self
        .auth
        .login(&dto.username, &dto.password)
        .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
    Ok(Json(tokens))
}
```

Without `.exception()`, you'd write:

```rust
let tokens = match self.auth.login(&dto.username, &dto.password) {
    Ok(t) => t,
    Err(e) => {
        return Err(HttpError::unauthorized("LOGIN_FAILED", e.message()));
    }
};
```

The `.exception()` version saves 4 lines and keeps the happy path visible.
