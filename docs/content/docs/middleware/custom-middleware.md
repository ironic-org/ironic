---
title: Custom Middleware
description: Register middleware at global, controller, or route level — including with macro-based controllers.
---

# Custom Middleware

Apply middleware with `#[middleware]` at the controller or route level.

## Controller-level

```rust
use ironic::prelude::*;

#[controller("/examples")]
#[middleware(LoggingMiddleware)]
#[middleware(AuthMiddleware)]
#[derive(Injectable)]
struct ExampleController {
    service: Arc<ExampleService>,
}

#[get]
async fn list(&self) -> Result<Json<Vec<Example>>, HttpError> {
    Ok(Json(self.service.list()))
}
```

## Route-level

```rust
#[controller("/examples")]
#[derive(Injectable)]
struct ExampleController {
    service: Arc<ExampleService>,
}

#[get("/{id}")]
#[middleware(LoggingMiddleware)]
#[middleware(RateLimitMiddleware::new(100))]
async fn get(&self, #[param] id: u64) -> Result<Json<Example>, HttpError> {
    Ok(Json(self.service.get(id)?))
}
```

## Using `#[guard]` for auth

```rust
#[controller("/examples")]
#[guard(RequireAuthenticated<User>)]
#[derive(Injectable)]
struct ExampleController {
    service: Arc<ExampleService>,
}

#[get("/{id}")]
#[guard(RequireAccess<User>::new("admin"))]
async fn get(&self, #[param] id: u64) -> Result<Json<Example>, HttpError> {
    Ok(Json(self.service.get(id)?))
}
```

**Key difference:** Middleware wraps the entire request. Guards run between middleware and the handler — use `#[guard]` for auth, `#[middleware]` for cross-cutting concerns.
