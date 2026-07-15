---
title: Interceptors
description: Wrap every request around handler execution — measure timing, transform responses, add headers, and enforce cross-cutting logic that needs access to the full request/response lifecycle.
---

# Interceptors

## What you'll learn

- Where interceptors sit in the request pipeline
- Implement the `Interceptor` trait to run code before and after the handler
- Build a timing interceptor, an envelope wrapper, and a header injector
- Register interceptors at global, controller, route, and attribute levels
- Understand execution order and the interceptor-vs-middleware trade-off

---

## The pipeline

Every HTTP request flows through a fixed sequence of stages. Interceptors sit **after guards** but **wrap** extraction, pipes, and the handler itself:

```
  Request ─► Middleware ─► Guards ─►╔═══════════════════════════╗──► Middleware unwind ─► Response
                                    ║    Interceptors            ║
                    global ─► controller ─► route                ║
                                   ╔══════════════════════════════╝
                                   ║   before  ┌──────────┐
                                   ╠══════════►│ Extract  │
                                   ║            │   Pipe   │
                                   ║   handler  │ Handler  │
                                   ║   ◄────────│          │
                                   ╚════════════════════════
                    route  ◄── controller ◄── global   (after)
```

**Key insight:** Interceptors form an onion. Code before `next.run(ctx)` executes on the way in; code after it runs on the way out. If an interceptor short-circuits (returns without calling `next`), the handler is never reached.

> **Interceptors vs Middleware vs Guards:** Middleware is the outermost wrapper — it sees the raw request first and can short-circuit everything. Guards decide access (allow/deny) and run before interceptors. Interceptors are the innermost wrapper — they have access to extracted parameters, can inspect the response body, and are ideal for per-route cross-cutting logic.

---

## The `Interceptor` trait

```rust
use ironic::{
    Interceptor, InterceptorNext, PipelineFuture,
    RequestContext, HttpError, FrameworkResponse,
};

pub struct MyInterceptor;

impl Interceptor for MyInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            // ── BEFORE the next stage ──
            let log_path = context.request().uri().path().to_owned();
            let start = std::time::Instant::now();

            // INVOKE the next interceptor (or the handler)
            let result = next.run(context).await;

            // ── AFTER the next stage ──
            let elapsed = start.elapsed();
            println!("{log_path} took {elapsed:?}");

            result
        })
    }
}
```

`InterceptorNext::run(ctx)` is a one-shot call. You **must** call it (unless you intend to short-circuit), and you can only call it once. Use it as the pivot point between your "before" and "after" logic.

All interceptors must be `Send + Sync + 'static`, so they can be stored in `Arc`s and shared across threads.

---

## Before/after handler pattern

Every interceptor follows the same skeleton:

```rust
impl Interceptor for MyInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            // 1. Inspect or mutate context before the handler
            //    e.g. insert data into context extensions

            let mut response = next.run(context).await?;

            // 2. Inspect or mutate the response after the handler
            //    e.g. add headers, wrap body, log status

            Ok(response)
        })
    }
}
```

**Context is `&'a mut RequestContext`** — you can read the incoming URI, headers, and method; insert extensions for downstream extractors; and read route metadata before calling `next`. After `next` returns, you have the full `FrameworkResponse` to inspect or transform.

**Accessing route metadata** from within an interceptor:

```rust
let metadata = context.route_metadata();
let ttl = metadata.and_then(|m| m.get::<CacheMetadata>()).cloned();
```

This is how the built-in `CacheInterceptor` decides whether caching applies to a route.

---

## Working examples

### 1. TimingInterceptor

Measure how long every handler takes and log it:

```rust
use std::time::Instant;
use ironic::{
    Interceptor, InterceptorNext, PipelineFuture,
    RequestContext, FrameworkResponse, HttpError,
};

pub struct TimingInterceptor;

impl Interceptor for TimingInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let path = context.request().uri().path().to_owned();
            let method = context.request().method().clone();
            let start = Instant::now();

            let response = next.run(context).await?;

            let duration = start.elapsed();
            let status = response.status().as_u16();
            println!("{method} {path} → {status} [{duration:?}]");

            Ok(response)
        })
    }
}
```

### 2. EnvelopeInterceptor

Wrap every JSON response in a `{data, meta, timestamp}` envelope:

```rust
use ironic::{
    Interceptor, InterceptorNext, PipelineFuture,
    RequestContext, FrameworkResponse, HttpError, HttpStatus,
};
use serde_json::{json, Value};

pub struct EnvelopeInterceptor;

impl Interceptor for EnvelopeInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let response = next.run(context).await?;

            let is_json = response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.starts_with("application/json"));

            if !is_json {
                return Ok(response);
            }

            let body = response.body().as_bytes().to_vec();
            let inner: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

            let envelope = json!({
                "data": inner,
                "meta": {
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status": response.status().as_u16(),
                }
            });

            let bytes = serde_json::to_vec(&envelope).unwrap_or_default();
            Ok(FrameworkResponse::bytes(response.status(), bytes))
        })
    }
}
```

> **Note:** The `chrono` crate is not bundled with Ironic. Add it to your `Cargo.toml` or replace it with `std::time::SystemTime`.

### 3. HeadersInterceptor

Add custom response headers to every request:

```rust
use ironic::{
    Interceptor, InterceptorNext, PipelineFuture,
    RequestContext, FrameworkResponse, HttpError,
};
use http::{HeaderName, HeaderValue};

pub struct HeadersInterceptor {
    name: HeaderName,
    value: HeaderValue,
}

impl HeadersInterceptor {
    pub fn new(name: &'static str, value: &'static str) -> Self {
        Self {
            name: HeaderName::from_static(name),
            value: HeaderValue::from_static(value),
        }
    }
}

impl Interceptor for HeadersInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let mut response = next.run(context).await?;
            response
                .headers_mut()
                .insert(self.name.clone(), self.value.clone());
            Ok(response)
        })
    }
}
```

---

## Registration levels

Interceptors can be attached at four levels of granularity. Each level adds to the chain — they do not replace each other.

### Global (applies to all routes)

```rust
FrameworkApplication::builder()
    .interceptor(TimingInterceptor)
    .interceptor(HeadersInterceptor::new("X-Powered-By", "Ironic"))
    .build().await?;
```

### Controller (all routes on one controller)

```rust
ControllerDefinition::new::<UserController>("/users", provider)?
    .interceptor(TimingInterceptor)
    .interceptor(HeadersInterceptor::new("X-Controller", "users"))
    .route(route);
```

### Route (single endpoint)

```rust
RouteDefinition::new(HttpMethod::GET, "/:id", "get_user", handler)?
    .interceptor(TimingInterceptor)
    .interceptor(HeadersInterceptor::new("X-Route", "get_user"));
```

### Attribute macro

On a controller struct (applies to every route method in that controller):

```rust
#[controller("/items")]
#[use_interceptor(TimingInterceptor)]
#[use_interceptor(HeadersInterceptor::new("X-Controller", "items"))]
impl ItemsController { /* ... */ }
```

On a single route method:

```rust
#[get("/:id")]
#[use_interceptor(TimingInterceptor)]
async fn get(&self, #[param] id: u64) -> Result<String, HttpError> {
    Ok(id.to_string())
}
```

---

## Execution order

Interceptors chain from outermost to innermost on the way in, and reverse on the way out:

```
global-before → controller-before → route-before → [extract → pipe → handler]
                                                         │
global-after  ← controller-after  ← route-after  ◄──────┘
```

Concretely, if you register `TimingInterceptor` globally and `EnvelopeInterceptor` on a route:

```
TimingInterceptor::before   (start clock)
  EnvelopeInterceptor::before
    ── handler executes ──
  EnvelopeInterceptor::after    (wraps in envelope)
TimingInterceptor::after    (logs elapsed time)
```

**Global interceptors always run first and finish last.** Route interceptors are innermost — closest to the handler.

---

## Interceptor vs Middleware vs Guard

|                     | Middleware        | Guard             | Interceptor             |
|---------------------|-------------------|-------------------|-------------------------|
| Runs                | First             | Second            | Third                   |
| Can short-circuit   | Yes               | Yes (via Deny)    | Yes                     |
| Can read response   | Yes               | No (only decide)  | Yes                     |
| Can transform body  | Yes               | No                | Yes                     |
| Access to metadata  | Limited           | Limited           | Full (route_metadata)   |
| Best for            | CORS, rate limits | Auth, permissions | Logging, envelopes, headers |

> **Rule of thumb:** Use middleware for platform-level concerns (CORS, rate limiting). Use guards for authorization. Use interceptors for everything that needs the full request/response lifecycle and route awareness.

---

## Try it yourself

1. Create a `TimingInterceptor` that logs `{method} {path} → {status} [{duration}]`
2. Register it globally — verify every route gets timed
3. Create a `PowerByInterceptor` that adds `X-Powered-By: Ironic` to every response
4. Register it at the controller level — verify only that controller's routes get the header
5. Apply `#[use_interceptor(TimingInterceptor)]` on a single route method
6. Send a request and confirm execution order: global → controller → route

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Forgetting to call `next.run(ctx)` | The handler is never reached; the response hangs |
| Calling `next.run(ctx)` twice | `InterceptorNext` is consumed on first call — won't compile |
| Mutating response body after `next` returns an error | Check the `Result` before transforming the response |
| Registering an expensive interceptor on every route | Scope interceptors at the right level (route vs global) |
| Accessing `context` after `next.run` and expecting pre-handler state | The handler (or downstream interceptors) may have mutated context |

## What you learned

- [x] Interceptors wrap extraction, pipes, and the handler
- [x] Implement `Interceptor` with `intercept(&self, ctx, next) -> PipelineFuture`
- [x] `next.run(ctx)` is the pivot: code before it is "pre-handler", code after is "post-handler"
- [x] Register at global, controller, route, or attribute level
- [x] Execution order: global → controller → route → handler → route → controller → global
- [x] Use interceptors for timing, response envelopes, and headers
- [x] Choose interceptors over middleware when you need route metadata or response body access
