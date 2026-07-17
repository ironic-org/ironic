---
title: Overview
description: Intercept and transform every request — tracing, logging, authentication, and more. Understand the pipeline, execution order, and registration levels.
---

# Middleware Overview

## What you'll learn

- The middleware pipeline: what runs before your handlers and why
- How execution order works and why it matters
- The three registration levels: global, controller, and route
- Every built-in middleware and when to use each one

Middleware sits **between the raw HTTP layer and your handler**, inspecting or transforming every request and response. It is the backbone of cross-cutting concerns — tracing, logging, auth, rate limiting — everything that applies across many routes without repeating code.

> **Why this matters:** Without middleware, you would duplicate auth checks, CORS headers, and request logging in every handler. Middleware gives you a composable, ordered pipeline that runs automatically — write it once, enforce it everywhere.

---

## Pipeline

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

The execution order is:

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

## Registration Levels

### Global
Applied to every request in the application. Registered on the application builder:

```rust
use ironic::prelude::*;

let app = FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(MyMiddleware)
    .platform(AxumAdapter::new())
    .build().await.unwrap();
```

Global middleware is the **outermost** layer — it runs first on the way in and last on the way out.

### Controller
Applied to all routes within a controller. Registered on the controller definition:

```rust
ControllerDefinition::new::<UserController>("/users", provider)
    .unwrap()
    .middleware(AuthMiddleware)   // all /users/* routes
    .route(get_user)
    .route(create_user);
```

### Route
Applied to a single route. Registered on the route definition:

```rust
RouteDefinition::new(HttpMethod::GET, "/admin/dashboard", "dashboard", handler)
    .unwrap()
    .middleware(AdminOnlyAuth);   // only this route
```

---

## Built-in Middleware

| Middleware | What it does | Default |
|---|---|---|
| [`RequestTracing`](./request-tracing) | Adds `x-request-id`, creates a tracing span with method + URI | Auto-registered |
| [`RequestLogging`](./request-logging) | Logs method, URI, status, body sizes, and duration as structured tracing events | Auto-registered (opt-out via `.without_request_logging()`) |
| `SecurityHeadersMiddleware` | HSTS, CSP, X-Frame-Options, Referrer-Policy | Feature: `security-headers` |
| `CorsMiddleware` | Handles preflight, sets `Access-Control-*` headers | Feature: `security-cors` |
| `RateLimitMiddleware` | IP-based sliding window rate limiting, returns 429 | Feature: `security-rate-limit` |
| `CsrfMiddleware` | Sets CSRF cookie, validates `x-csrf-token` header | Feature: `security-csrf` |

Security middleware requires the corresponding feature flag and is documented in the [Security](../http-api/security) section.

---

## Common mistakes

| Mistake | Fix |
|---|---|
| Forgetting to call `next.run(context)` | The handler never executes — always `.await` on `next.run()` unless you intentionally short-circuit |
| Not calling `.await` on `next.run(context)` | `MiddlewareNext::run` returns a `Future` — use `.await` or the pipeline never advances |
| Blocking synchronously inside an async `handle` | Use `tokio::task::spawn_blocking` for CPU-heavy work |
| Registering middleware after `.build()` | Middleware must be registered during construction; `FrameworkApplication` is immutable after build |
| Assuming route middleware runs before controller middleware | Controller wraps route — controller runs first on the way in |

## What you learned

- [x] Middleware wraps the handler pipeline in a stack — global → controller → route → handler
- [x] Call `next.run(context)` to advance; skip it to short-circuit
- [x] Three registration levels: global (builder), controller, route
- [x] `RequestTracing` and `RequestLogging` are auto-registered by default
- [x] Execution order is global → controller → route → handler → route → controller → global
