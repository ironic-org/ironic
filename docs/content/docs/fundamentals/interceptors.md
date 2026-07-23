---
title: Interceptors
description: Hook into the request-response cycle with pre-handler and post-handler interceptors.
---

# Interceptors

Interceptors run before and after your route handler. They can transform requests, modify responses, add logging, attach metadata, or short-circuit the pipeline.

## How interceptors work

```
Request → Guard → Interceptor (pre) → Handler → Interceptor (post) → Response
                    │
                    ▼
               Error → short-circuit
```

## Interceptor trait

```rust
pub trait Interceptor: Send + Sync + 'static {
    async fn intercept(&self, context: &mut InterceptorContext) -> Result<(), HttpError>;
}
```

## When interceptors run

| Phase | Timing | Common uses |
|-------|--------|-------------|
| **Pre-handler** | After guards, before the handler | Request validation, logging, auth injection |
| **Post-handler** | After the handler, before response | Response transformation, header injection |

## Pre-handler interceptor

Runs after guards but before the handler. Can short-circuit by returning an error:

```rust
struct RequestLoggingInterceptor;

impl Interceptor for RequestLoggingInterceptor {
    async fn intercept(&self, ctx: &mut InterceptorContext) -> Result<(), HttpError> {
        tracing::info!(
            method = %ctx.request().method(),
            path = %ctx.request().uri(),
            "incoming request"
        );
        Ok(())
    }
}
```

## Post-handler interceptor

Runs after the handler produces a response. Can modify the response:

```rust
struct VersioningInterceptor;

impl Interceptor for VersioningInterceptor {
    async fn intercept(&self, ctx: &mut InterceptorContext) -> Result<(), HttpError> {
        if let Some(response) = ctx.response_mut() {
            response.headers_mut().insert(
                "X-API-Version",
                HeaderValue::from_static("1.0"),
            );
        }
        Ok(())
    }
}
```

## Registering interceptors

```rust
#[routes]
impl UserController {
    #[interceptor(RequestLoggingInterceptor)]
    #[get("/users")]
    async fn list(&self) -> Json<Vec<User>> {
        // interceptor runs before this handler
    }
}
```

Controller-level interceptors apply to all routes:

```rust
#[controller("/api")]
#[interceptor(VersioningInterceptor)]
struct ApiController {
    // version header is added to all routes
}
```

## Interceptor context

The `InterceptorContext` provides:

| Method | Description |
|--------|-------------|
| `request()` | Read the HTTP request |
| `request_mut()` | Modify the HTTP request |
| `response()` | Read the response (post-handler only) |
| `response_mut()` | Modify the response (post-handler only) |
| `container()` | Access the DI container |
| `set_ext(key, value)` | Attach metadata for downstream use |

## Short-circuiting

Return `Err(HttpError)` to stop pipeline execution:

```rust
impl Interceptor for RateLimitInterceptor {
    async fn intercept(&self, ctx: &mut InterceptorContext) -> Result<(), HttpError> {
        if is_rate_limited(ctx.request()).await {
            return Err(HttpError::too_many_requests(
                "RATE_LIMIT_EXCEEDED",
                "Too many requests",
            ));
        }
        Ok(())
    }
}
```

## Comparing interceptors to other concepts

| Concept | Runs | Use case |
|---------|------|----------|
| Middleware | Before router (tower layer) | Cross-cutting concerns (CORS, compression) |
| Guards | After routing, before handler | Authorization |
| Interceptors | Around handler | Request/response transformation |
| Pipes | Inside handler parameters | Parameter transformation |
| Exception Filters | After handler error | Error response formatting |

## Testing interceptors

```rust
#[tokio::test]
async fn test_version_header() {
    let interceptor = VersioningInterceptor;
    let mut ctx = InterceptorContext::new(mock_request());

    assert!(interceptor.intercept(&mut ctx).await.is_ok());

    let response = ctx.response().unwrap();
    assert_eq!(response.headers()["X-API-Version"], "1.0");
}
```
