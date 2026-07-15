---
title: Middleware
description: Intercept and transform every request — logging, authentication, CORS, rate limiting, and more.
---

# Middleware

## What you'll learn

- The middleware pipeline: what runs before your handlers and why
- How to write custom middleware with the `Middleware` trait
- The three registration levels: global, controller, and route
- Every built-in middleware and when to use each one
- How order determines execution and why it matters

Middleware sits **between the raw HTTP layer and your handler**, inspecting or transforming every request and response. It is the backbone of cross-cutting concerns — tracing, auth, rate limiting, security headers — everything that applies across many routes without repeating code.

> **Why this matters:** Without middleware, you'd duplicate auth checks, CORS headers, and request logging in every handler. Middleware gives you a composable, ordered pipeline that runs automatically — write it once, enforce it everywhere.

---

## Section 1 — Concept

Every request flows through a chain of middleware *before* reaching your handler. When the handler returns, the response unwinds back through the same chain in reverse:

```
        REQUEST ↓
 ┌─────────────────────────┐
 │    Global Middleware     │ ← outermost
 │  ┌───────────────────┐   │
 │  │ Controller Middleware│ │
 │  │  ┌───────────────┐ │   │
 │  │  │ Route Middleware│ │  │
 │  │  │    ┌─────┐    │ │   │
 │  │  │    │Handler│   │ │   │
 │  │  │    └─────┘    │ │   │
 │  │  └───────────────┘ │   │
 │  └───────────────────┘   │
 └─────────────────────────┘
        RESPONSE ↑
```

At the core is the `Middleware` trait:

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
- `next` is a `MiddlewareNext<'a>` handle — call `next.run(context)` to proceed down the pipeline.
- `PipelineFuture<'a>` is `Pin<Box<dyn Future<Output = Result<FrameworkResponse, HttpError>> + Send + 'a>>`.

The pattern is always the same: do work **before** calling `next.run(context)`, then do work **after** it resolves.

---

## Section 2 — Building Custom Middleware

### Request Logger
Logs the method, path, and wall-clock duration for every request:

```rust
use std::time::Instant;
use ironic::prelude::*;

pub struct RequestLogger;

impl Middleware for RequestLogger {
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
                status = %response.status().as_u16(),
                duration_ms = elapsed.as_millis() as u64,
                "request completed"
            );
            Ok(response)
        })
    }
}
```

Key points:
- `Box::pin(async move { ... })` — every `handle` returns a boxed, pinned future.
- `next.run(context)` is the gateway to the rest of the pipeline; if you don't call it, the handler never runs.
- Work before `next.run` runs on the way **in**; work after runs on the way **out**.

### Auth Header Checker
Rejects requests missing an `x-api-key` header:

```rust
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

Returning `Err(HttpError::...)` short-circuits the pipeline — the response unwinds through any middleware that already ran.

---

## Section 3 — Registration Levels

### Global
Applied to every request in the application:

```rust
use ironic::{FrameworkApplication, AxumAdapter};

let app = FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(RequestLogger)                // first to run on request
    .middleware(SecurityHeadersMiddleware::new(
        SecurityHeadersConfig::default()
    ))
    .platform(AxumAdapter::new())
    .build()
    .await
    .unwrap();
```

Global middleware is the **outermost** layer — it runs first on the way in and last on the way out.

### Controller
Applied to all routes within a controller:

```rust
ControllerDefinition::new::<UserController>("/users", provider)
    .unwrap()
    .middleware(ApiKeyAuth)   // all /users/* routes need an API key
    .route(get_user_route)
    .route(create_user_route);
```

### Route
Applied to a single route:

```rust
RouteDefinition::new(HttpMethod::GET, "/admin/dashboard", "dashboard", handler)
    .unwrap()
    .middleware(AdminOnlyAuth)   // only this route
```

---

## Section 4 — Built-in Middleware

| Middleware | What it does | Constructor |
|---|---|---|
| `RequestTracing` | Adds `x-request-id`, creates a `tracing` span with method + URI | `RequestTracing::new()` — auto-registered |
| `SecurityHeadersMiddleware` | HSTS, CSP, X-Frame-Options: DENY, Referrer-Policy, and more | `SecurityHeadersMiddleware::new(SecurityHeadersConfig::default())` |
| `CorsMiddleware` | Handles preflight, sets `Access-Control-*` headers | `CorsMiddleware::new(CorsConfig::new().allowed_origins(origins))` |
| `RateLimitMiddleware` | IP-based sliding window rate limiting, returns 429 | `RateLimitMiddleware::new(backend, max_requests, window_secs)` |
| `CsrfMiddleware` | Sets CSRF cookie, validates `x-csrf-token` header on mutating requests | `CsrfMiddleware::new(CsrfConfig::new())` |

`RequestTracing` is automatically registered as global middleware by the framework. All other security middleware lives in `crate ironic-security` and requires the corresponding feature flag:

```toml
ironic = { features = ["security"] }
```

Individual security features if you need granular control:

| Middleware | Feature flag |
|---|---|
| `SecurityHeadersMiddleware` | `security-headers` |
| `CorsMiddleware` | `security-cors` |
| `RateLimitMiddleware` | `security-rate-limit` |
| `CsrfMiddleware` | `security-csrf` |

### Rate limit backends

`RateLimitMiddleware` accepts any backend implementing `RateLimitBackend`:

```rust
pub trait RateLimitBackend: Send + Sync {
    /// Returns Ok(()) if the request is allowed, or an error with the
    /// remaining cooldown if the limit is exceeded.
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
    ) -> RateLimitResult;
}
```

Two built-in backends are provided:

| Backend | Feature | Storage | Use case |
|---------|---------|---------|----------|
| `InMemoryRateLimiter` | `security-rate-limit` | In-process `HashMap` | Development, single-replica |
| `RedisRateLimiter` | `security-rate-limit` + `redis` | Redis INCR + EXPIRE | Production, multi-replica |

**InMemoryRateLimiter** is the default (no Redis needed):

```rust
use ironic::security::rate_limit::{InMemoryRateLimiter, RateLimitMiddleware};

let backend = Arc::new(InMemoryRateLimiter::new());
let middleware = RateLimitMiddleware::new(backend, 100, 60);
```

**RedisRateLimiter** for production deployments with multiple replicas,
using an atomic INCR + EXPIRE pipeline:

```rust
use ironic::security::rate_limit::{RedisRateLimiter, RateLimitMiddleware};

let backend = Arc::new(RedisRateLimiter::new(redis_client));
let middleware = RateLimitMiddleware::new(backend, 100, 60);
```

### Custom rate limit backend

Implement `RateLimitBackend` for any storage — PostgreSQL, DynamoDB, etc.:

```rust
use ironic::security::rate_limit::{RateLimitBackend, RateLimitResult};
use std::collections::HashMap;
use std::sync::Mutex;

struct SlidingWindowCounter {
    // (key, window_start) → count
    counts: Mutex<HashMap<(String, u64), u64>>,
}

impl SlidingWindowCounter {
    fn new() -> Self {
        Self { counts: Mutex::new(HashMap::new()) }
    }
}

impl RateLimitBackend for SlidingWindowCounter {
    async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
    ) -> RateLimitResult {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window_start = now - (now % window_secs);

        let mut counts = self.counts.lock().unwrap();

        // Clean old windows
        counts.retain(|(_, start), _| *start > now - window_secs);

        let entry = counts.entry((key.to_owned(), window_start)).or_insert(0);
        *entry += 1;

        if *entry <= max_requests {
            RateLimitResult::allowed(*entry)
        } else {
            let reset_at = window_start + window_secs;
            RateLimitResult::denied(*entry, reset_at)
        }
    }
}
```

### Rate limit response headers

When a request is rate-limited, the response includes:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Max requests in the window |
| `X-RateLimit-Reset` | Unix timestamp when the window resets |

Example response:

```
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Reset: 1734192000
```

The `X-RateLimit-Reset` value is a Unix epoch timestamp.  Clients can use it
to implement retry-after logic without additional state.

### Testing rate limiting

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_rate_limit_allows_within_limit() {
        let app = build_app_with_rate_limit(5, 60);
        for _ in 0..5 {
            let response = app
                .clone()
                .oneshot(Request::get("/").body(axum::body::Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_after_limit() {
        let app = build_app_with_rate_limit(3, 60);
        for _ in 0..3 {
            app.clone()
                .oneshot(Request::get("/").body(axum::body::Body::empty()).unwrap())
                .await
                .unwrap();
        }
        let response = app
            .clone()
            .oneshot(Request::get("/").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_rate_limit_headers() {
        let app = build_app_with_rate_limit(1, 60);
        let response = app
            .oneshot(Request::get("/").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(response.headers().contains_key("x-ratelimit-limit"));
        assert!(response.headers().contains_key("x-ratelimit-reset"));
    }

    #[tokio::test]
    async fn test_in_memory_backend() {
        let backend = InMemoryRateLimiter::new();
        let result = backend.check_rate_limit("test-key", 5, 60).await;
        assert!(result.is_allowed());
    }
}
```

---

## Section 5 — Ordering & Execution

The pipeline is a **stack**: each middleware wraps the one inside it. The execution order is:

```
REQUEST IN  →  global[0]  →  global[1]  →  controller[0]  →  route[0]  →  HANDLER
RESPONSE OUT ←  global[1]  ←  global[0]  ←  controller[0]  ←  route[0]  ←
```

When you register multiple middleware at the same level, the **first registered runs outermost**:

```rust
// FrameworkApplication::builder()
//    .middleware(A)   ← runs first on request, last on response
//    .middleware(B)   ← runs second on request, second-to-last on response
```

If `A` returns an error without calling `next.run(context)`, `B` and everything inside never executes — but `A`'s post-next code still runs.

**Guards and interceptors** sit between middleware and the handler: middleware → guards → interceptors → extraction → handler. A denied guard still unwinds through all upstream middleware.

---

## Common mistakes

| Mistake | Fix |
|---|---|
| Forgetting to call `next.run(context)` | The handler never executes — always `.await` on `next.run()` unless you intentionally short-circuit |
| Not calling `.await` on `next.run(context)` | `MiddlewareNext::run` returns a `Future` — use `.await` or the pipeline never advances |
| Blocking synchronously inside an async `handle` | Use `tokio::task::spawn_blocking` for CPU-heavy work |
| Adding security middleware without the feature flag | Add `ironic = { features = ["security"] }` to `Cargo.toml` |
| Registering middleware after `.build()` | Middleware must be registered during construction; `FrameworkApplication` is immutable after build |
| Assuming route middleware runs before controller middleware | Controller wraps route — controller runs first on the way in |

## Try it yourself

1. Write a `TimingMiddleware` that logs the request duration to `stdout` and register it globally
2. Create an `ApiVersionMiddleware` that reads an `Accept-Version` header and short-circuits with a 400 if it's missing
3. Register `RateLimitMiddleware::new(5, 60)` on a single route and confirm the 429 response after five requests
4. Chain three middlewares at the global level and use `tracing::info!` to verify execution order matches the documented pattern
5. Write a middleware that injects a `UserId` struct into `RequestContext` via `context.insert_extension(...)` and read it in a handler

## What you learned

- [x] Every middleware implements `handle(context, next) -> PipelineFuture`
- [x] `next.run(context)` advances the pipeline; skipping it short-circuits
- [x] Global, controller, and route-level registration compose into an ordered stack
- [x] `RequestTracing` is the only auto-registered middleware; all security middleware needs the `security` feature
- [x] Execution order is global → controller → route → handler → route → controller → global
- [x] Return `Err(HttpError::...)` to short-circuit from any middleware layer
