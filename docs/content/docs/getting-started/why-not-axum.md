---
title: Why not axum?
description: Ironic is built on axum. Here's how they relate and when to use each.
---

# Why not axum?

Ironic is **built on axum**, not a replacement for it. Axum is the platform adapter that runs Ironic applications. The question isn't "Ironic vs axum" — it's whether you want the additional structure Ironic provides on top.

## How they relate

```
┌──────────────────────────────┐
│          Ironic              │
│  Modules, DI, Controllers,  │
│  Auth, Config, Metrics, etc.│
├──────────────────────────────┤
│          Axum                │
│  Router, Extractors,         │
│  Tower middleware, HTTP/1.1  │
├──────────────────────────────┤
│         Tower / Hyper        │
│  Service, Layer, HTTP core   │
└──────────────────────────────┘
```

Ironic compiles down to axum's `Router` at runtime. Your controllers, middleware, and guards all become axum-compatible services and layers.

## When to use axum directly

Axum is the right choice when:

- You're building a **small API** with 1-5 endpoints
- You want **complete control** over every aspect of the HTTP stack
- You're adding a **single endpoint** to an existing non-Ironic project
- You prefer to **compose your own stack** from tower middleware

## When to use Ironic

Ironic adds value when:

- You have **10+ endpoints** organized into feature modules
- You need **dependency injection** to manage service/repository wiring
- You want **built-in auth, config, metrics, OpenAPI** without manual integration
- Your team benefits from a **standard project structure**
- You're building something that will **grow over time**

## What Ironic adds on top of axum

| Feature | Axum | Ironic |
|---------|------|--------|
| Routing | Manual router setup | Auto-discovered via `#[controller]` macros |
| State sharing | `Extension` or `State` | Full DI container with scoped providers |
| Middleware | Tower layers | Guard/Interceptor/ExceptionFilter pipeline |
| Configuration | Manual env parsing | Typed, validated, layered config with hot-reload |
| Auth | Manual | JWT, OAuth2, sessions with guard decorators |
| OpenAPI | Third-party | Auto-generated from route definitions |
| Testing | `tower::Service` tests | TestModule, in-process client, fluent assertions |
| CLI | None | `ironic new`, `ironic gen`, `ironic doctor` |

## Can I use axum middleware with Ironic?

Yes. Ironic exposes the underlying axum `Router` through the adapter, so you can apply any tower/axum middleware directly:

```rust
AxumAdapter::new()
    .configure_router(|router| {
        router.layer(tower_http::compression::CompressionLayer::new())
    })
    .build(app.compile())
```

## Performance overhead

Ironic adds minimal overhead on top of axum:

- **DI resolution**: ~200ns per provider (cached after first resolution)
- **Controller dispatch**: ~150ns per request (single vtable call)
- **Middleware pipeline**: ~50ns per layer (tower layer overhead)
- **Total overhead vs raw axum**: ~1µs per request (negligible for 99% of use cases)

See [Benchmarks](/docs/getting-started/benchmarks) for detailed numbers.
