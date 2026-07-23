---
title: HTTP Transport
description: RESTful HTTP APIs — controllers, routes, middleware, error handling, and more.
---

# HTTP Transport

The HTTP transport is Ironic's primary protocol. It provides a full-featured REST API framework built on Axum with controllers, route definitions, middleware pipelines, extractors, guards, interceptors, and structured error handling.

## Controllers & Routes

Controllers group related routes under a common path prefix:

```rust
use ironic::*;

#[controller("/users")]
struct UserController;

#[routes]
impl UserController {
    #[get("/")]
    async fn list(&self) -> Json<Vec<User>> {
        // ...
    }

    #[post("/")]
    async fn create(&self, body: JsonBody<CreateUser>) -> Json<User> {
        // ...
    }
}
```

## Request Pipeline

Every request flows through a configurable pipeline:

```
Request → Middleware → Guards → Interceptors → Handler → Response
                          ↓
                    Exception Filters ← Error
```

- **Middleware**: Cross-cutting concerns (logging, auth, CORS, rate limiting)
- **Guards**: Authorization checks before handler execution
- **Interceptors**: Pre/post handler hooks for validation, transformation
- **Exception Filters**: Centralized error handling
- **Pipes**: Parameter transformation and validation

## Extractors

Extractors pull data from the request:

| Extractor | Description |
|-----------|-------------|
| `JsonBody<T>` | JSON request body |
| `FormBody<T>` | URL-encoded form body |
| `PathParameter` | Path parameters (`/users/:id`) |
| `QueryParameters` | Query string parameters |
| `HeaderParameter` | Request headers |

## Responses

Handlers return responses via `IntoResponse` implementations:

- `Json<T>` — JSON response with content-type
- `Html<T>` — HTML response
- `Redirect` — 301/302 redirect
- `HttpResponse` — Full control over status, headers, body

## Middleware

Built-in middleware layers:

- **Logging**: Request/response logging with `MetricsLayer`
- **CORS**: Cross-Origin Resource Sharing
- **CSRF**: Cross-Site Request Forgery protection
- **Rate Limiting**: In-memory or Redis-backed
- **Security Headers**: HSTS, CSP, X-Frame-Options, etc.
- **Compression**: Gzip, Brotli, Zstd
- **Exception Filters**: Centralized error handling

## Error Handling

Structured errors with `HttpError`:

```rust
return Err(HttpError::bad_request("VALIDATION_FAILED", "Invalid input"));
return Err(HttpError::unauthorized("AUTH_INVALID_TOKEN", "Token expired"));
return Err(HttpError::internal("DB_CONNECTION_FAILED", "Database unavailable"));
```

## Additional Topics

- [Content Negotiation](/docs/http-api/content-negotiation)
- [Response Serialization](/docs/http-api/response-serialization)
- [Streaming Responses](/docs/http-api/streaming-responses)
- [Exception Filters](/docs/http-api/exception-filters)
- [Validation Pipes](/docs/http-api/validation-pipes)
- [API Versioning](/docs/http-api/api-versioning)
- [Multipart Uploads](/docs/advanced/multipart)
- [Static Files](/docs/advanced/static-files)
