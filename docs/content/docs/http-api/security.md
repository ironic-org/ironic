---
title: Security
description: Protect your API with CORS, rate limiting, CSRF tokens, and security headers — all feature-flagged for zero overhead.
---

# Security

## What you'll learn

- Enable CORS to control which websites can call your API
- Add rate limiting to prevent abuse
- Protect forms with CSRF tokens
- Set security headers (HSTS, CSP, X-Frame-Options)
- Enable compression for faster responses

Enable in `Cargo.toml`:

```toml
ironic = { features = ["security", "compression"] }
```

---

## CORS (Cross-Origin Resource Sharing)

Control which websites can access your API:

```rust
use ironic::security::cors::{CorsConfig, CorsMiddleware};

let cors = CorsConfig::new()
    .allow_origin("https://myapp.com")
    .allow_methods(["GET", "POST"])
    .allow_headers(["Content-Type", "Authorization"])
    .max_age(3600);

FrameworkApplication::builder()
    .platform(AxumAdapter::new().configure_router(|r| {
        r.layer(CorsMiddleware::layer(cors));
    }))
    .build().await.unwrap();
```

> **Why this matters:** Without CORS, any website can call your API with the user's cookies. CORS restricts it to trusted origins.

---

## Rate Limiting

Prevent abuse by limiting requests per client:

```rust
use ironic::security::rate_limit::{InMemoryRateLimiter, RateLimitConfig, RateLimitMiddleware};

let config = RateLimitConfig::new()
    .max_requests(100)                           // 100 requests...
    .per_window(std::time::Duration::from_secs(60)); // ...per minute

AxumAdapter::new().configure_router(|r| {
    r.layer(RateLimitMiddleware::layer(config));
});
```

When a client exceeds the limit: **429 Too Many Requests**.

> **Production note:** The in-memory limiter is fine for single-server deployments. For multi-server setups, enable the `redis` feature for a distributed rate limiter.

---

## CSRF Protection

Protect HTML forms from cross-site request forgery:

```rust
use ironic::security::csrf::{CsrfConfig, CsrfMiddleware};

let csrf = CsrfConfig::new()
    .cookie_name("csrf-token")
    .header_name("X-CSRF-Token");

AxumAdapter::new().configure_router(|r| {
    r.layer(CsrfMiddleware::layer(csrf));
});
```

The server sets a CSRF cookie. The client reads it and sends it back in the `X-CSRF-Token` header. If they don't match → **403 Forbidden**.

---

## Security Headers

Add browser security headers automatically:

```rust
use ironic::security::security_headers::SecurityHeadersMiddleware;

AxumAdapter::new().configure_router(|r| {
    r.layer(SecurityHeadersMiddleware::layer());
});
```

This adds:

| Header | Value | What it does |
|--------|-------|--------------|
| `Strict-Transport-Security` | `max-age=31536000` | Forces HTTPS |
| `X-Content-Type-Options` | `nosniff` | Prevents MIME sniffing |
| `X-Frame-Options` | `DENY` | Prevents clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Stops reflected XSS |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limits referrer info |
| `Content-Security-Policy` | `default-src 'self'` | Blocks inline scripts |

---

## Compression

Make responses smaller (and faster):

```rust
FrameworkApplication::builder()
    .platform(AxumAdapter::new().compression())
    .build().await.unwrap();
```

Supports **gzip**, **brotli**, and **zstd** — automatically pick the best one the client supports.

---

## Try it yourself

1. Add CORS that only allows `https://example.com`
2. Add rate limiting: 5 requests per second
3. Enable all security headers
4. Test: call from a different origin → should be blocked
5. Test: make 6 rapid requests → should get 429

## Common mistakes

| Mistake | Fix |
|---------|-----|
| CORS `*` in production | Never use `*` — specify exact origins |
| Rate limit too aggressive | Test with real client behavior before setting limits |
| CSRF on pure API (no forms) | CSRF is for form-based auth; skip it if you use JWT with Authorization header |
| Compression on tiny responses | Compressing 10-byte responses is wasteful; it's fine for normal API responses |

## What you learned

- [x] CORS controls cross-origin access
- [x] Rate limiting prevents abuse (100 req/min)
- [x] CSRF protects form submissions
- [x] Security headers harden your API
- [x] Compression speeds up responses
