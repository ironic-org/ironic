---
title: Security and production defaults
description: Request limits, CORS, security headers, rate limits, CSRF, and secret handling.
---

# Security and production defaults

The Axum adapter buffers at most 1 MiB per request and applies a 30-second end-to-end timeout.
Override these only after measuring the endpoint:

```rust
use std::time::Duration;
use ironic::AxumAdapter;

let adapter = AxumAdapter::new()
    .request_body_limit(2 * 1024 * 1024)
    .request_timeout(Duration::from_secs(10));
```

## Built-in security middleware

Ironic provides four security middleware modules under the `security` feature umbrella.
Each module implements the `Middleware` trait from the Ironic request pipeline and is
registered through `RouteDefinition::middleware()`, `ControllerDefinition::middleware()`,
or on a compiled application's global pipeline.

### Enabling features

```toml
# Cargo.toml
ironic = { features = ["security"] }
```

Or enable individual modules:

```toml
ironic = { features = [
    "security-cors",      # CORS middleware
    "security-headers",   # Security response headers
    "security-rate-limit",# In-memory rate limiting (requires dep:redis)
    "security-csrf",      # CSRF token validation (requires dep:uuid)
] }
```

### CORS

`CorsConfig` and `CorsMiddleware` handle cross-origin requests:

```rust
use ironic::{CorsConfig, CorsMiddleware, HttpMethod};

let config = CorsConfig::new()
    .allowed_origins(["https://app.example.com"])
    .allowed_methods([HttpMethod::GET, HttpMethod::POST])
    .allowed_headers(["content-type", "authorization"])
    .allow_credentials(true)
    .max_age(3600);

let middleware = CorsMiddleware::new(config);

// Register on a route, controller, or globally:
// RouteDefinition::new(...).middleware(middleware)
```

Do not use a wildcard origin with credentials; enumerate trusted origins and methods.
The middleware sets `Access-Control-Allow-Origin`, `Access-Control-Allow-Methods`,
`Access-Control-Allow-Headers`, `Access-Control-Allow-Credentials`, and
`Access-Control-Max-Age` headers. Preflight `OPTIONS` requests are handled automatically.

### Security headers

`SecurityHeadersConfig` and `SecurityHeadersMiddleware` set recommended response headers:

```rust
use ironic::SecurityHeadersConfig;

let config = SecurityHeadersConfig::new()
    .hsts("max-age=31536000; includeSubDomains")
    .csp("default-src 'self'")
    .x_content_type_options("nosniff")
    .x_frame_options("DENY")
    .referrer_policy("strict-origin-when-cross-origin");

let middleware = SecurityHeadersMiddleware::new(config);
// RouteDefinition::new(...).middleware(middleware)
```

Disable individual headers when termination happens at a proxy:

```rust
SecurityHeadersConfig::new()
    .disable_hsts()        // TLS terminated at proxy
    .disable_csp()         // CSP managed externally
    .x_content_type_options("nosniff");
```

### Rate limiting

`RateLimiter` provides in-memory sliding-window rate limiting:

```rust
use ironic::{
    RateLimiter, RateLimitConfig, RateLimitMiddleware,
};

let limiter = RateLimiter::new(100, 60); // 100 requests per 60-second window
// RouteDefinition::new(...).middleware(RateLimitMiddleware::new(limiter))
```

The rate limiter is keyed by the calling IP address. Custom key functions can be
provided for authenticated principals. On exceeding the limit, the middleware returns
`429 Too Many Requests`.

### CSRF

`CsrfConfig` and `CsrfMiddleware` validate CSRF tokens from request headers against
signed cookies:

```rust
use ironic::{CsrfConfig, CsrfMiddleware};

let config = CsrfConfig::new()
    .cookie_name("csrf-token")
    .header_name("x-csrf-token");

let middleware = CsrfMiddleware::new(config);
// RouteDefinition::new(...).middleware(middleware)
```

The middleware generates a token on `GET` requests (stored in a signed cookie) and
validates the token from the configured header on state-changing requests.

## Response compression

Enable gzip, brotli, or zstd compression with a single builder call:

```rust
use ironic::AxumAdapter;

let adapter = AxumAdapter::new().compression();
```

The compression layer respects `Accept-Encoding` from the client and only compresses
responses larger than 1 KB with a compressible content type.

## Secrets

Load credentials from environment variables or a secret manager, wrap typed values in `Secret<T>`,
and never include exposed values in logs, errors, tracing fields, panic messages, or generated
configuration files. Rotate credentials independently of application releases.
