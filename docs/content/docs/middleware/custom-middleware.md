---
title: Custom Middleware
description: Build your own middleware — logging, authentication, header injection, and any cross-cutting concern. Register at global, controller, or route level.
---

# Custom Middleware

## What you'll learn

- The `Middleware` trait and the before/after pattern
- How to build a timing logger and an auth checker
- The three registration levels: global, controller, route
- How to short-circuit the pipeline with errors
- How to pass request-scoped data via extensions

---

## The Middleware trait

Every middleware implements the `Middleware` trait:

```rust
pub trait Middleware: Send + Sync + 'static {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a>;
}
```

- `context` carries the incoming request plus extensions you set.
- `next` advances the pipeline — call `next.run(context).await` to proceed.
- The return type is a boxed, pinned future.

The pattern is always: do work **before** calling `next.run(context)`, then do work **after** it resolves.

---

## Example: Request Timing

Logs the method, path, and wall-clock duration for every request:

```rust
use std::time::Instant;
use ironic::prelude::*;

pub struct TimingMiddleware;

impl Middleware for TimingMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let method = context.request().method().clone();
            let path = context.request().uri().path().to_owned();
            let start = Instant::now();
            let response = next.run(context).await?;
            let elapsed = start.elapsed();
            tracing::info!(
                method = %method,
                path = %path,
                status = response.status().as_u16(),
                duration_ms = elapsed.as_millis() as u64,
                "timing"
            );
            Ok(response)
        })
    }
}
```

---

## Example: API Key Auth

Rejects requests missing an `x-api-key` header:

```rust
use ironic::prelude::*;

pub struct ApiKeyAuth;

impl Middleware for ApiKeyAuth {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let has_key = context
                .request()
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .is_some();
            if !has_key {
                return Err(HttpError::unauthorized(
                    "MISSING_API_KEY",
                    "x-api-key header is required",
                ));
            }
            next.run(context).await
        })
    }
}
```

Returning `Err(HttpError::...)` short-circuits the pipeline — upstream middleware still unwinds, but the handler never runs.

---

## Registration Levels

### Global — applied to every request

Register on the application builder. Global middleware is the outermost layer:

```rust
let app = FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(TimingMiddleware)
    .middleware(ApiKeyAuth)
    .platform(AxumAdapter::new())
    .build().await.unwrap();
```

Order matters: `TimingMiddleware` wraps `ApiKeyAuth`, which wraps the rest of the pipeline.

### Controller — applied to all routes in a controller

Register on the controller definition:

```rust
ControllerDefinition::new::<AdminController>("/admin", provider)
    .unwrap()
    .middleware(ApiKeyAuth)           // every /admin/* route
    .route(dashboard_route)
    .route(users_route);
```

### Route — applied to a single route

Register on the route definition:

```rust
RouteDefinition::new(
    HttpMethod::POST,
    "/admin/delete-all",
    "delete_all",
    handler,
)?
.middleware(AdminOnlyAuth)            // only this route
.middleware(ConfirmationMiddleware);  // also only this route
```

### Mixing levels

All three levels compose into a single ordered stack:

```rust
// Global (outermost)
FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(RequestLogging)       // global: runs first/last
    .platform(AxumAdapter::new())
    .build().await.unwrap();

// Controller (middle)
ControllerDefinition::new::<ApiController>("/api", provider)
    .unwrap()
    .middleware(ApiKeyAuth)           // controller: runs after global, before route
    .route(some_route);

// Route (innermost)
RouteDefinition::new(HttpMethod::GET, "/api/admin", "admin", handler)?
    .middleware(AdminOnlyAuth);       // route: runs just before the handler
```

---

## Passing data via extensions

Use `context.insert_extension()` and `context.extension::<T>()` to pass request-scoped data between middleware and handlers:

```rust
#[derive(Clone)]
struct UserId(String);

pub struct AuthMiddleware;

impl Middleware for AuthMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let token = context.request()
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            // Validate token, extract user...
            context.insert_extension(UserId("user-42".into()));
            next.run(context).await
        })
    }
}

// In your handler
fn handler(context: RequestContext) -> impl IntoFrameworkResponse {
    if let Some(user) = context.extension::<UserId>() {
        format!("hello {}", user.0)
    } else {
        "unknown user".to_string()
    }
}
```

---

## Testing custom middleware

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ironic::prelude::*;

    #[tokio::test]
    async fn timing_records_duration() {
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/test",
            "test",
            handler_fn(|_controller: Arc<()>, _args| async move {
                Ok::<_, HttpError>(FrameworkResponse::empty(HttpStatus::OK))
            }),
        ).unwrap();

        let mut app = build_http_application_with_overrides(
            &CompiledApplicationGraph::default(),
            Vec::new(),
        ).unwrap()
        .middleware(TimingMiddleware);

        let request = FrameworkRequest::new(
            HttpMethod::GET,
            "/test".parse().unwrap(),
            HeaderMap::new(),
            Vec::new(),
        );
        let mut ctx = RequestContext::new(request);
        let result = app.execute(&app.routes()[0], &mut ctx).await;
        assert!(result.is_ok());
    }
}
```

---

## Common mistakes

| Mistake | Fix |
|---|---|
| Forgetting to call `next.run(context)` | The handler never executes — always `.await` on `next.run()` unless you intentionally short-circuit |
| Not calling `.await` on `next.run()` | `MiddlewareNext::run` returns a `Future` — use `.await` |
| Blocking synchronously in an async `handle` | Use `tokio::task::spawn_blocking` for CPU-heavy work |
| Registering middleware after `.build()` | Middleware must be registered during construction |
| Using `context.extension::<T>()` without calling `insert_extension` first | The extension will return `None` — insert before the handler runs |

## What you learned

- [x] Implement `Middleware` trait with `handle(context, next) -> PipelineFuture`
- [x] Work before `next.run()` executes on the way in; after, on the way out
- [x] Return `Err(HttpError::...)` to short-circuit the pipeline
- [x] Three registration levels: global (builder), controller, route
- [x] Pass request-scoped data via `context.insert_extension()`
- [x] Test middleware by building a minimal app and sending requests
