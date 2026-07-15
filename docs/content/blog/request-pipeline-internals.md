---
title: "Inside the Ironic request pipeline — from HTTP to handler and back"
description: "A step-by-step walkthrough of how a request flows through middleware, guards, interceptors, extraction, pipes, and into your handler — with real source code and a concrete execution trace."
date: "2026-07-15"
author: "Ironic Team"
---

# Inside the Ironic request pipeline — from HTTP to handler and back

Every framework has a request pipeline. Most of them describe it with arrows and boxes in a diagram. Here, we'll walk through the actual source code — line by line, function by function — so you can see exactly what happens between the moment a request hits the server and the moment a response goes out.

The pipeline is a stack of onion layers. A request enters from the outside, passes through in order, and exits in reverse. Each layer gets a mutable reference to `RequestContext`, can read or mutate anything on it, and can decide to short-circuit the whole thing.

Here's the order:

```
HTTP Request
    │
    ▼
Middleware (global → controller → route)
    │
    ▼
Guards (global → controller → route)
    │
    ▼
Interceptors (global → controller → route)
    │
    ▼
Parameter Extraction
    │
    ▼
Parameter Pipes (global → controller → route)
    │
    ▼
Handler
    │
    ▼
    (unwind: interceptors → middleware in reverse)
    │
    ▼
HTTP Response
```

Let's trace it from the top.

---

## The entry point: `CompiledHttpApplication::execute()`

It all starts at `route.rs:658-667`. When the router matches a request to a compiled route, it calls this:

```rust
pub async fn execute(
    &self,
    route: &CompiledRoute,
    context: &mut RequestContext,
) -> Result<FrameworkResponse, HttpError> {
    if context.extension::<crate::RequestScope>().is_none() {
        context.insert_extension(self.container.request_scope());
    }
    super::pipeline::execute(self, route, context).await
}
```

There are two things happening here. First, **auto-injection of the DI scope**. The method checks whether the `RequestContext` already carries a `RequestScope` in its extensions map. If it doesn't, it creates one by calling `self.container.request_scope()` and inserts it. This is how every handler gets access to per-request scoped dependencies — the scope is transparently attached before anything else runs.

Second, it delegates to `pipeline::execute()`. That's where the real machinery lives.

---

## `pipeline::execute()` — the orchestrator

In `pipeline.rs:223-253`, the outer execute function does three things: stamp route metadata onto the context, kick off the middleware chain, and handle any errors that come back:

```rust
pub(crate) async fn execute(
    application: &CompiledHttpApplication,
    route: &CompiledRoute,
    context: &mut RequestContext,
) -> Result<FrameworkResponse, HttpError> {
    context.set_route_metadata(route.metadata().clone());
    let state = ExecutionState { application, route };
    match run_middleware(&state, 0, context).await {
        Ok(response) => Ok(response),
        Err(error) => {
            // Route-level filters first, then global filters
            // ...
        }
    }
}
```

The `ExecutionState` struct at line 186 is a tiny bundle:

```rust
struct ExecutionState<'a> {
    application: &'a CompiledHttpApplication,
    route: &'a CompiledRoute,
}
```

It holds the two things every pipeline stage needs — the app (for global components) and the route (for route-specific components). This gets threaded through every function call.

---

## Middleware: the outer onion layer

`run_middleware()` at line 255 is where the chain begins:

```rust
fn run_middleware<'a>(
    state: &'a ExecutionState<'a>,
    index: usize,
    context: &'a mut RequestContext,
) -> PipelineFuture<'a> {
    if let Some(middleware) = middleware_at(state, index) {
        middleware.handle(
            context,
            MiddlewareNext {
                state,
                index: index + 1,
            },
        )
    } else {
        run_guards(state, context)
    }
}
```

This is a recursive iteration disguised as a loop. Each middleware gets a `MiddlewareNext` handle that captures the current state and an incremented index. When the middleware calls `next.run(context)`, it triggers the next middleware — or falls through to guards if there are none left.

`middleware_at()` (line 339) looks up components by interleaving global and route-level middleware:

```rust
fn middleware_at<'a>(
    state: &'a ExecutionState<'a>,
    index: usize,
) -> Option<&'a Arc<dyn Middleware>> {
    let global = &state.application.pipeline().middleware;
    global
        .get(index)
        .or_else(|| state.route.pipeline().middleware.get(index - global.len()))
}
```

Global middleware runs first (lower indices), then controller middleware, then route-level. Same pattern for guards and interceptors — global first, then per-route.

The `Middleware` trait itself is straightforward (`pipeline.rs:30-37`):

```rust
pub trait Middleware: Send + Sync + 'static {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a>;
}
```

A middleware implementation does its work, calls `next.run(context)`, and potentially does more work after. The `MiddlewareNext` is a move-only handle — you can only call `.run()` once, which prevents accidental re-entry.

This is the onion. A middleware that logs timing could look like:

```rust
fn handle<'a>(&'a self, context: &'a mut RequestContext, next: MiddlewareNext<'a>) -> PipelineFuture<'a> {
    Box::pin(async move {
        let start = Instant::now();
        let result = next.run(context).await;  // ← runs inner layers
        let elapsed = start.elapsed();
        println!("Request took {:?}", elapsed);
        result  // ← unwinding begins
    })
}
```

The `PipelineFuture` type at line 9 is a boxed, pinned future:

```rust
pub type PipelineFuture<'a> =
    Pin<Box<dyn Future<Output = Result<FrameworkResponse, HttpError>> + Send + 'a>>;
```

Every stage returns this same type. The whole pipeline is one big async call tree.

---

## Guards: yes or no, no partial credit

If all middleware passes, `run_guards()` at line 273 picks up:

```rust
fn run_guards<'a>(
    state: &'a ExecutionState<'a>,
    context: &'a mut RequestContext,
) -> PipelineFuture<'a> {
    Box::pin(async move {
        let count = guard_count(state);
        for index in 0..count {
            match guard_at(state, index)
                .expect("guard index is in bounds")
                .can_activate(context)
                .await?
            {
                GuardDecision::Allow => {}
                GuardDecision::Deny => {
                    return Err(HttpError::forbidden(
                        "RF_HTTP_GUARD_DENIED",
                        "Access to this route was denied",
                    ));
                }
            }
        }
        run_interceptor(state, 0, context).await
    })
}
```

Guards are evaluated sequentially. The `Guard` trait (`line 40-43`) has a single method:

```rust
pub trait Guard: Send + Sync + 'static {
    fn can_activate<'a>(&'a self, context: &'a mut RequestContext) -> GuardFuture<'a>;
}
```

The return type is `GuardFuture`, which resolves to `GuardDecision` (`line 21-27`):

```rust
pub enum GuardDecision {
    Allow,
    Deny,
}
```

If any guard returns `Deny`, the pipeline stops immediately with a forbidden error. No interceptors, no extraction, no handler. But here's the interesting part: because guards run *inside* the middleware onion, if a guard denies, the error propagates back *through the middleware layers in reverse order*. The test at line 721 confirms this — when a guard denies, the events show middleware unwinding:

```
"global-middleware-before",
"controller-middleware-before",
"route-middleware-before",
"global-guard",              ← denial happens here
"route-middleware-after",    ← exit through middleware in reverse
"controller-middleware-after",
"global-middleware-after",
```

No interceptors, no handler. The onion unwinds cleanly.

---

## Interceptors: the inner onion

If guards pass, `run_interceptor()` at line 298 takes over:

```rust
fn run_interceptor<'a>(
    state: &'a ExecutionState<'a>,
    index: usize,
    context: &'a mut RequestContext,
) -> PipelineFuture<'a> {
    if let Some(interceptor) = interceptor_at(state, index) {
        interceptor.intercept(
            context,
            InterceptorNext {
                state,
                index: index + 1,
            },
        )
    } else {
        Box::pin(async move {
            let controller = route_scope(context)?
                .resolve_key(state.route.controller())
                .await?;
            state.route.invoke_handler(controller, context).await
        })
    }
}
```

The pattern is identical to middleware — each interceptor gets an `InterceptorNext` handle, and the last one falls through to the handler. The `Interceptor` trait (`line 46-53`):

```rust
pub trait Interceptor: Send + Sync + 'static {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a>;
}
```

At the base of the interceptor chain, when there are no more interceptors, the function resolves the controller from the DI container (using the `RequestScope` that was auto-injected at the start) and calls `invoke_handler()`.

---

## Where the handler lives: `CompiledRoute`

Every route stores its handler as an `Arc<dyn ErasedHandler>`. You can see this in `route.rs:99`:

```rust
pub struct RouteDefinition {
    method: HttpMethod,
    path: String,
    handler_name: &'static str,
    parameters: Vec<ParameterDefinition>,
    handler: Arc<dyn ErasedHandler>,  // ← erased at definition time
    pipeline: PipelineComponents,
    metadata: RouteMetadata,
}
```

The `ErasedHandler` trait in `handler.rs:51-54`:

```rust
pub trait ErasedHandler: Send + Sync + 'static {
    fn call(&self, controller: ProviderValue, arguments: HandlerArguments) -> HandlerFuture;
}
```

When you call `handler_fn(your_function)`, it wraps your closure in a `HandlerFn<C, F, Fut, R>` struct that implements `ErasedHandler`. At call time (`handler.rs:68-79`), it downcasts the `ProviderValue` back to `Arc<C>`, calls your closure, and boxes the result.

---

## Extraction and pipes

`CompiledRoute::invoke_handler()` in `route.rs:520-537` is where parameters get built:

```rust
pub(crate) async fn invoke_handler(
    &self,
    controller: ProviderValue,
    context: &mut RequestContext,
) -> Result<FrameworkResponse, HttpError> {
    let mut arguments = Vec::with_capacity(self.parameters.len());
    for parameter in &self.parameters {
        let mut value = parameter.extractor.extract(context).await?;
        for pipe in &parameter.pipes {
            value = pipe.transform(value, context).await?;
        }
        arguments.push(value);
    }
    self.handler
        .call(controller, HandlerArguments::new(arguments))
        .await
}
```

For each declared parameter, the extractor runs first — it pulls data from the request (path, headers, query, body) and returns an `ExtractedValue`, which is just `Box<dyn Any + Send>` (`extract.rs:8`). Then each pipe in the chain transforms the value. `ParameterPipe` (`pipeline.rs:56-66`):

```rust
pub trait ParameterPipe: Send + Sync + 'static {
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        context: &'a mut RequestContext,
    ) -> PipeFuture<'a>;
}
```

Pipes run in order: global pipes first (prepended at `route.rs:645`), then controller pipes, then route-level pipes. Each wraps the type-erased value, downcasts it, applies the transformation, and re-boxes it. If any pipe fails, the error propagates up through interceptors and middleware, and no handler code runs. The test at line 760 confirms this — when a pipe fails, the handler is never called, but the interceptor and middleware after-hooks do fire.

---

## Concrete trace: `GET /users/:id`

Let's trace a real request through the actual source. Imagine this route definition:

```rust
RouteDefinition::new(
    HttpMethod::GET,
    "/:id",
    "find_user",
    handler_fn(|_controller: Arc<UserController>, mut arguments| async move {
        let id: u64 = arguments.take(0)?;
        Ok(Json(User { id, name: "Ada".into() }))
    }),
)
.unwrap()
.parameter(PathParameter::<u64>::new("id"));
```

Mounted on controller `/users`. Here's what happens step by step:

**1. Entry.** `CompiledHttpApplication::execute()` at `route.rs:658` receives the matched `CompiledRoute` and a `&mut RequestContext`. The context already contains the parsed request (method, URI, headers, body). If no `RequestScope` extension is present, one is auto-created from the container.

**2. Route metadata.** `pipeline::execute()` at `pipeline.rs:228` calls `context.set_route_metadata(route.metadata().clone())`, making route-level metadata (like cache directives or OpenAPI tags) available to downstream components.

**3. Middleware.** `run_middleware(state, 0, context)` fires. Each middleware gets `context: &mut RequestContext` and a `MiddlewareNext { state, index: index + 1 }`. Global middleware runs first, then controller, then route. If any middleware returns `Err(...)`, the error propagates immediately — no further middleware, guards, or anything else runs. The exception filter catch block in `execute()` at line 231 kicks in.

**4. Guards.** `run_guards(state, context)` iterates all guards. Each guard's `can_activate(context)` returns `GuardDecision::Allow` or `GuardDecision::Deny`. All guards must allow. On deny, the error is `HttpError::forbidden("RF_HTTP_GUARD_DENIED", ...)` at line 287, and the middleware onion unwinds in reverse.

**5. Interceptors.** `run_interceptor(state, 0, context)` runs the interceptor chain. At the bottom, `route_scope(context)` at line 327 retrieves the `RequestScope` extension that was injected in step 1. The controller is resolved via `scope.resolve_key(route.controller())`.

**6. Extraction.** `CompiledRoute::invoke_handler()` at `route.rs:520` iterates over parameters. For our route, there's one: a `PathParameter<u64>` named `"id"`. The extractor at `extract.rs:46-62` reads the path parameter, parses it as `u64`, and returns `Box::new(42_u64)` as `ExtractedValue`. If parsing fails, the error is `RF_HTTP_INVALID_PATH_PARAMETER`.

**7. Pipes.** If pipes are registered, each one transforms the value in sequence. For this route, no pipes are registered, so the value passes through as-is.

**8. Handler.** The erased handler's `.call()` method (`handler.rs:68`) downcasts the `ProviderValue` to `Arc<UserController>`, builds `HandlerArguments` from the extracted values, and invokes your closure. Your handler calls `arguments.take::<u64>(0)` to pull out the parsed ID, does its work, and returns `Json(User { ... })`.

**9. Response conversion.** `IntoFrameworkResponse` converts the return value into a `FrameworkResponse` with proper status code and headers.

**10. Unwind.** The response propagates back through each interceptor's after-hook, then each middleware's after-hook — in reverse order. The test at line 685 documents the exact event order:

```
global-middleware-before
controller-middleware-before
route-middleware-before
global-guard
controller-guard
route-guard
global-interceptor-before
controller-interceptor-before
route-interceptor-before
extract
pipe
handler
route-interceptor-after     ← unwind begins
controller-interceptor-after
global-interceptor-after
route-middleware-after
controller-middleware-after
global-middleware-after
```

---

## Error handling and the exception filter cascade

Errors don't just propagate blindly. `pipeline::execute()` at line 230-251 has a two-tier filter system:

```rust
match run_middleware(&state, 0, context).await {
    Ok(response) => Ok(response),
    Err(error) => {
        // Route-level filters (most specific first)
        if let Some(result) = route.pipeline().exception_filters.catch(&error, &filter_ctx) {
            return result;
        }
        // Global-level filters
        if let Some(result) = application.pipeline().exception_filters.catch(&error, &filter_ctx) {
            return result;
        }
        Err(error)
    }
}
```

Route-level filters get first crack at the error. If none handle it, global filters get a chance. If no filter catches it, the original error propagates to the caller. The test at line 1155 confirms that route filters take precedence over global ones. This means you can define narrow recovery logic on specific routes while keeping broad fallback handlers at the application level.

---

## The shared state: `&mut RequestContext`

One detail worth calling out: every stage receives `context: &'a mut RequestContext`. There's no hidden state, no thread-local, no magical `HttpContext.current()`. The context is the single mutable reference that threads through the entire pipeline. Anything you attach via `context.insert_extension(...)` in middleware is visible to guards, interceptors, extractors, and the handler. This is how auth middleware can set a `User` extension that a guard reads, how a logging interceptor can access route metadata, and how the DI scope persists from entry to handler.

The `RequestScope` extension itself is the clearest example — inserted once at `route.rs:663-664`, retrieved at `pipeline.rs:327-335` when the controller needs to be resolved, and dropped when the context is dropped at the end of the request.

---

## Summary

The Ironic request pipeline is a depth-first async call tree built from three composable layers (middleware, guards, interceptors) plus typed extraction and validation. The key design decisions:

- **Onion model with `MiddlewareNext` / `InterceptorNext` handles** — each layer decides when to call the next one, and the call stack naturally unwinds in reverse when errors occur.
- **`RequestContext` as the single mutable thread** — every component sees the same context, and extensions accumulate as the request flows through stages.
- **Auto-injected DI scope** — `CompiledHttpApplication::execute()` ensures a `RequestScope` exists before the pipeline starts, so per-request dependencies are always available.
- **Type-erased handlers** — `ErasedHandler` / `HandlerFn` / `handler_fn()` erase the concrete `Arc<C>` type so routes can be stored in a homogeneous collection, with downcasting at invocation time.
- **Extractors as reusable building blocks** — `PathParameter`, `HeaderParameter`, `QueryParameters`, `JsonBody` each implement `ParameterExtractor` and return `Box<dyn Any + Send>`, composable with typed `ParameterPipe` chains.

The pipeline tests in `pipeline.rs:684-982` are an excellent reference — they instrument every stage with recording components and assert the exact event order. If you want to understand the pipeline's behavior under every failure condition, those tests tell the whole story.
