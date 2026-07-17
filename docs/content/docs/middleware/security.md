---
title: Security Middleware
description: Security headers, CORS, rate limiting, and CSRF protection — feature-gated middleware provided by Ironic.
---

# Security Middleware

## Enabling

```toml
ironic = { features = ["security"] }

# Or individually:
ironic = { features = ["security-headers"] }
ironic = { features = ["security-cors"] }
ironic = { features = ["security-rate-limit"] }
ironic = { features = ["security-csrf"] }
```

## SecurityHeadersMiddleware

Sets browser security headers on every response — HSTS, CSP, X-Frame-Options, and more.

```rust
use ironic::security::security_headers::{SecurityHeadersConfig, SecurityHeadersMiddleware};

FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(SecurityHeadersMiddleware::new(SecurityHeadersConfig::default()))
    .platform(AxumAdapter::new())
    .build().await.unwrap();
```

## CorsMiddleware

Controls which origins can access your API.

```rust
use ironic::security::cors::{CorsConfig, CorsMiddleware};

FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(CorsMiddleware::new(
        CorsConfig::new()
            .allow_origin("https://myapp.com")
            .allow_methods(["GET", "POST"])
    ))
    .platform(AxumAdapter::new())
    .build().await.unwrap();
```

## RateLimitMiddleware

Limits requests per client IP. Returns 429 when exceeded.

```rust
use ironic::security::rate_limit::{InMemoryRateLimiter, RateLimitConfig, RateLimitMiddleware};
use std::sync::Arc;
use std::time::Duration;

FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(RateLimitMiddleware::new(
        Arc::new(InMemoryRateLimiter::new()),
        RateLimitConfig::new()
            .max_requests(100)
            .per_window(Duration::from_secs(60)),
    ))
    .platform(AxumAdapter::new())
    .build().await.unwrap();
```

For multi-replica deployments, use `RedisRateLimiter` (requires `redis` feature):

```rust
use ironic::security::rate_limit::RedisRateLimiter;

let backend = Arc::new(RedisRateLimiter::new(redis_client));
```

## CsrfMiddleware

Protects form submissions with synchronizer tokens. Returns 403 on mismatch.

```rust
use ironic::security::csrf::{CsrfConfig, CsrfMiddleware};

FrameworkApplication::builder()
    .module(AppModule::definition())
    .middleware(CsrfMiddleware::new(
        CsrfConfig::new()
            .cookie_name("csrf-token")
            .header_name("X-CSRF-Token"),
    ))
    .platform(AxumAdapter::new())
    .build().await.unwrap();
```

> Skip CSRF if your API uses JWT in `Authorization` headers — it is already immune.
