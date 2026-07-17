---
title: Overview
description: Middleware reference — all middleware provided by Ironic, how to enable them, and where to register them.
---

# Middleware Overview

Ironic provides the following middleware out of the box. Some are auto-registered, others require a feature flag and manual registration.

## Auto-registered

These run on every request with no setup:

- **`RequestTracing`** — Adds `x-request-id` header, creates a tracing span per request.
- **`RequestLogging`** — Logs method, URI, status, body sizes, duration as structured tracing events.

## Feature-gated

These require a feature flag and manual `.middleware(...)` registration:

| Middleware | Feature | Registration |
|---|---|---|
| `SecurityHeadersMiddleware` | `security-headers` | `.middleware(SecurityHeadersMiddleware::new(...))` |
| `CorsMiddleware` | `security-cors` | `.middleware(CorsMiddleware::new(...))` |
| `RateLimitMiddleware` | `security-rate-limit` | `.middleware(RateLimitMiddleware::new(...))` |
| `CsrfMiddleware` | `security-csrf` | `.middleware(CsrfMiddleware::new(...))` |
| `AuthenticationMiddleware` | `auth` | `.middleware(AuthenticationMiddleware::new(...))` |

## Registration points

```rust
// Global — applies to every request
FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(MyMiddleware)
    .platform(AxumAdapter::new())
    .build().await.unwrap();

// Controller — applies to all routes in a controller
#[controller("/admin")]
#[middleware(AdminAuth)]
struct AdminController;

// Route — applies to a single route
#[controller("/admin")]
struct AdminController;

#[routes]
impl AdminController {
    #[delete("/users/{id}")]
    #[middleware(AdminOnly)]
    async fn delete_user(&self, #[param] id: u64) -> Result<Json<()>, HttpError> {
        // ...
    }
}
```

## Middleware execution order

```
REQUEST IN → global → controller → route → HANDLER → route → controller → global → RESPONSE OUT
```
