---
title: Error Handling
description: Handle errors gracefully with structured error codes, exception filters, and the error envelope.
---

# Error Handling

Ironic provides a structured error system built around `HttpError`, exception filters, and a standard error envelope format.

## The error envelope

All error responses follow a consistent JSON format:

```json
{
    "error": "VALIDATION_FAILED",
    "message": "Email must be a valid email address",
    "statusCode": 400
}
```

## HttpError

The primary error type is `HttpError`:

```rust
pub struct HttpError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}
```

### Creating errors

```rust
// 400 Bad Request
HttpError::bad_request("VALIDATION_FAILED", "Invalid input");

// 401 Unauthorized
HttpError::unauthorized("AUTH_INVALID_TOKEN", "Token expired");

// 403 Forbidden
HttpError::forbidden("AUTH_FORBIDDEN", "Insufficient permissions");

// 404 Not Found
HttpError::not_found("NOT_FOUND", "User not found");

// 409 Conflict
HttpError::conflict("CONFLICT_DUPLICATE", "Email already exists");

// 429 Too Many Requests
HttpError::too_many_requests("RATE_LIMIT_EXCEEDED", "Slow down");

// 500 Internal Server Error
HttpError::internal("DB_CONNECTION_FAILED", "Database unavailable");

// 503 Service Unavailable
HttpError::service_unavailable("MAINTENANCE", "Under maintenance");
```

### Returning errors from handlers

```rust
#[routes]
impl UserController {
    #[get("/users/{id}")]
    async fn get(&self, id: PathParameter<i32>) -> Result<Json<User>, HttpError> {
        let user = self.service.find_by_id(*id).await
            .map_err(|_| HttpError::internal("DB_ERROR", "Failed to query user"))?
            .ok_or_else(|| HttpError::not_found("NOT_FOUND_USER", "User not found"))?;

        Ok(Json(user))
    }
}
```

## Exception filters

Exception filters let you customize error responses:

```rust
struct NotFoundFilter;

impl ExceptionFilter for NotFoundFilter {
    fn catch(&self, error: &HttpError, _ctx: &FilterContext) -> Option<HttpResponse> {
        if error.status == StatusCode::NOT_FOUND {
            Some(HttpResponse::new(404)
                .header("content-type", "text/html")
                .body("<h1>404 - Page Not Found</h1>"))
        } else {
            None  // Pass to next filter
        }
    }
}
```

Registering filters:

```rust
impl Module for AppModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .exception_filter(Arc::new(NotFoundFilter))
            .build()
    }
}
```

## Error codes

Use predefined error codes from `ironic::error_codes::codes`:

```rust
use ironic::error_codes::codes;

return Err(HttpError::not_found(codes::NOT_FOUND_USER, "User not found"));
```

| Category | Example codes |
|----------|--------------|
| Auth | `AUTH_INVALID_CREDENTIALS`, `AUTH_INVALID_TOKEN`, `AUTH_FORBIDDEN` |
| Validation | `VALIDATION_FAILED` |
| Not Found | `NOT_FOUND`, `NOT_FOUND_USER` |
| Conflict | `CONFLICT_DUPLICATE` |
| Rate Limit | `RATE_LIMIT_EXCEEDED` |
| Internal | `INTERNAL_ERROR`, `INTERNAL_DATABASE` |

## Backtrace support

When the `backtrace` feature is enabled, `HttpError::internal()` captures a backtrace:

```toml
[dependencies]
ironic = { version = "1.0", features = ["backtrace"] }
```

```rust
let error = HttpError::internal("DB_ERROR", "Connection failed");
// error.backtrace contains the call stack
```

## Best practices

- **Use status-code-specific constructors** — `bad_request()`, `not_found()`, etc. rather than `HttpError::new()`
- **Define custom error codes** — Create constants for your domain errors
- **Use `Result<T, HttpError>` as return type** — The `?` operator converts errors automatically
- **Catch errors at module boundaries** — Use exception filters for the last mile of error formatting
- **Log internal errors** — Use `tracing::error!` for 5xx errors with full context
