---
title: Interceptors
description: Run code before and after your handler — timing, response envelopes, header injection, and cross-cutting logic.
---

# Interceptors

## What is an Interceptor?

An interceptor is a piece of code that **wraps** your route handler. It runs some logic *before* the handler, calls the handler, then runs more logic *after* the handler. Think of it like an onion layer — you have code on the outside, the handler on the inside.

**Simple analogy:** A waiter takes your order (before), gives it to the kitchen (handler), then brings the food back to your table (after). The interceptor is the waiter — it can check your order, time how long the kitchen takes, or add a garnish before serving.

**Real example — measure how long every endpoint takes:**

```rust
use std::time::Instant;
use ironic::{Interceptor, InterceptorNext, PipelineFuture, RequestContext};

pub struct TimingInterceptor;

impl Interceptor for TimingInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let start = Instant::now();           // ← BEFORE

            let response = next.run(context).await?;  // ← HANDLER RUNS HERE

            let elapsed = start.elapsed();        // ← AFTER
            tracing::info!("Request took {:?}", elapsed);

            Ok(response)
        })
    }
}
```

The key method is `intercept()`. You get:
- `context` — the request context (headers, URI, method, extensions)
- `next` — call `next.run(context)` to invoke the next interceptor (or the handler)

Code before `next.run()` executes inbound. Code after it executes outbound. If you skip `next.run()`, the handler never runs — you've short-circuited the request.

## When to use an Interceptor vs other tools

| You want to... | Use |
|---|---|
| Check if the user is allowed (yes/no) | `#[guard]` |
| Run code before AND after the handler | `#[interceptor]` |
| Wrap every request (even errors) | `#[middleware]` |
| Catch errors from the handler | `#[exception]` |
| Transform/extract a parameter | `#[decorator]` |

**Rule of thumb:** Interceptors are for cross-cutting logic that needs to see both the request *and* the response. Timing, response envelopes, header injection — these are classic interceptor use cases.

## Registering interceptors

### Controller-level

Apply `#[interceptor(...)]` on the controller struct — every route inherits it:

```rust
#[controller("/blogs")]
#[interceptor(TimingInterceptor)]
#[derive(Injectable)]
struct BlogsController;
```

### Route-level

Apply on individual handler methods:

```rust
#[controller("/blogs")]
#[derive(Injectable)]
struct BlogsController;

#[routes]
impl BlogsController {
    #[get("/{id}")]
    #[interceptor(TimingInterceptor)]
    async fn show(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
        // timing interceptor runs only for this route
    }
}
```

### Global

For interceptors that apply to every route in the application:

```rust
FrameworkApplication::builder()
    .interceptor(TimingInterceptor)
    .build().await.unwrap();
```

## Execution order

Interceptors chain from outermost to innermost:

```
global-before → controller-before → route-before → [handler]
global-after  ← controller-after  ← route-after  ← [handler]
```

If you register `TimingInterceptor` globally and `EnvelopeInterceptor` on a route:

```
TimingInterceptor::before   (start clock)
  EnvelopeInterceptor::before
    — handler runs —
  EnvelopeInterceptor::after    (wrap in envelope)
TimingInterceptor::after    (log elapsed)
```

## Working examples

### Response envelope wrapper

Wrap every JSON response in `{ data, meta }`:

```rust
use ironic::{Interceptor, InterceptorNext, PipelineFuture, RequestContext, FrameworkResponse};

pub struct EnvelopeInterceptor;

impl Interceptor for EnvelopeInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let response = next.run(context).await?;
            let body = response.body().as_bytes().to_vec();
            let envelope = serde_json::json!({
                "data": serde_json::from_slice::<serde_json::Value>(&body).unwrap_or_default(),
                "meta": { "status": response.status().as_u16() }
            });
            Ok(FrameworkResponse::json(
                response.status(),
                &envelope,
            )?)
        })
    }
}
```

### Custom header injector

Add a header to every response from a controller:

```rust
pub struct PoweredByInterceptor;

impl Interceptor for PoweredByInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let mut response = next.run(context).await?;
            response.headers_mut().insert(
                "X-Powered-By".parse().unwrap(),
                "Ironic".parse().unwrap(),
            );
            Ok(response)
        })
    }
}
```

## Common mistakes

| Mistake | Fix |
|---|---|
| Forgetting to call `next.run(ctx)` | The handler is never reached — response hangs |
| Calling `next.run(ctx)` twice | `InterceptorNext` is consumed on first call — won't compile |
| Modifying response after an error | Check the `Result` before transforming the response |

## What you learned

- [x] Interceptors wrap the handler — code before `next.run()` is inbound, code after is outbound
- [x] Use `#[interceptor]` on controllers or route methods, or `.interceptor()` globally
- [x] Execution order: global → controller → route → handler → route → controller → global
- [x] Interceptors are for cross-cutting logic that needs both request and response access
