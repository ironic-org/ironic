---
title: Rate Limit Key Customization
description: Rate limit by user ID, API key, or tenant — not just IP address.
---

# Rate Limit Key Customization

## What is it?

By default, `RateLimitMiddleware` identifies clients by their IP address. In production, you often need per-user, per-token, or per-tenant limits. `.key_resolver()` lets you customize how the rate limit key is derived.

## How to use

```rust
RateLimitMiddleware::new(100, 60)
    .key_resolver(|ctx| {
        ctx.extension::<Claims>()
            .map(|c| c.sub.clone())
            .unwrap_or_else(|| "anonymous".into())
    })
```

## Common resolvers

```rust
// By user ID from JWT claims
.key_resolver(|ctx| {
    ctx.extension::<Claims>()
        .map(|c| c.sub.clone())
        .unwrap_or_else(|| "anon".into())
})

// By API key from header
.key_resolver(|ctx| {
    ctx.request().headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("no-key")
        .to_owned()
})

// By tenant from path
.key_resolver(|ctx| {
    ctx.request().uri().path()
        .split('/').nth(2)
        .unwrap_or("default")
        .to_owned()
})
```

## Why not just IP?

Corporate NAT, VPNs, and mobile networks share IPs across users. Rate limiting by IP can block legitimate users who share an exit node. Always use the most specific identifier available.
