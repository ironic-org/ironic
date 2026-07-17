---
title: Security
description: Protect your API — security headers, CORS, rate limiting, CSRF, and compression.
---

# Security

Security middleware is now documented in the dedicated [Middleware section](../middleware/overview). This page provides a quick reference.

## Feature flags

```toml
ironic = { features = ["security"] }
ironic = { features = ["compression"] }
```

## Quick reference

| Middleware | What it does | Where to register | Docs |
|---|---|---|---|
| `SecurityHeadersMiddleware` | HSTS, CSP, X-Frame-Options | `.middleware(...)` | [Middleware → Security](../middleware/security#securityheadersmiddleware) |
| `CorsMiddleware` | Cross-origin access control | `.middleware(...)` | [Middleware → Security](../middleware/security#corsmiddleware) |
| `RateLimitMiddleware` | IP-based rate limiting | `.middleware(...)` | [Middleware → Security](../middleware/security#ratelimitmiddleware) |
| `CsrfMiddleware` | CSRF token protection | `.middleware(...)` | [Middleware → Security](../middleware/security#csrfmiddleware) |
| `CompressionLayer` | gzip/brotli/zstd compression | `.compression()` on adapter | [HTTP & API → Compression](./compression) |

## Quick start

```rust
use ironic::prelude::*;
use ironic::security::{
    cors::{CorsConfig, CorsMiddleware},
    rate_limit::{InMemoryRateLimiter, RateLimitConfig, RateLimitMiddleware},
    csrf::{CsrfConfig, CsrfMiddleware},
    security_headers::{SecurityHeadersConfig, SecurityHeadersMiddleware},
};
use std::sync::Arc;
use std::time::Duration;

FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(SecurityHeadersMiddleware::new(SecurityHeadersConfig::default()))
    .middleware(CorsMiddleware::new(
        CorsConfig::new().allow_origin("https://myapp.com"),
    ))
    .middleware(RateLimitMiddleware::new(
        Arc::new(InMemoryRateLimiter::new()),
        RateLimitConfig::new()
            .max_requests(100)
            .per_window(Duration::from_secs(60)),
    ))
    .middleware(CsrfMiddleware::new(CsrfConfig::new()))
    .platform(AxumAdapter::new().compression())
    .build().await.unwrap();
```

> **Note:** Security middleware implements the `Middleware` trait — register via `.middleware(...)`, not `.configure_router(|r| r.layer(...))`.
