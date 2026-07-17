---
title: "Exception Filters — typed error handling with scope precedence"
description: "How Ironic's exception filter chain converts pipeline errors into custom responses — with scoped filter dispatch, fallback-to-default, and a concrete NotFoundFilter example."
date: "2026-07-15"
author: "Ironic Team"
---

# Exception Filters — typed error handling with scope precedence

When a handler panics, a guard denies access, or parameter extraction fails, the framework surfaces an `HttpError`. Without exception filters, that error travels untouched through the middleware onion and lands as a raw JSON response — serviceable, but never what you want your users to see.

Exception filters let you intercept errors mid-flight and convert them into polished, context-aware responses. They're typed (you match on `HttpError::status()` and `HttpError::code()`), scoped (route beats controller beats global), and chainable (FIFO, first to catch wins). The whole dispatch lives in two files: `exception_filter.rs` (89 lines) and `pipeline.rs` (lines 230–251).

---

## The `ExceptionFilter` trait

At `crates/ironic-http/src/exception_filter.rs:30`, the trait is minimal:

```rust
pub trait ExceptionFilter: Send + Sync + 'static {
    fn catch(
        &self,
        error: &HttpError,
        context: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError>;
}
```

Return `Ok(response)` to handle the error — the framework sends that response instead of the raw error. Return `Err(error)` to pass — the next filter in the chain gets a shot. You can even return a _different_ `HttpError` from `Err(...)`, which is how filters transform one error class into another (e.g., mapping a generic 500 into a 422 when the real cause was validation).

The `HttpError` struct (`error.rs:7`) carries three fields: `status: HttpStatus`, `code: &'static str`, and `message: String`. The `code` is a stable identifier like `"RF_HTTP_GUARD_DENIED"` — perfect for `match` arms that don't break when message text changes.

## `FilterContext` — route metadata at error time

When an error fires, you're no longer inside the handler. You've lost access to extracted parameters, the DI scope, and the request body. What you _do_ have is `FilterContext` (`exception_filter.rs:9`):

```rust
pub struct FilterContext {
    route_metadata: RouteMetadata,
}
```

`RouteMetadata` (`route.rs:24`) is a type-indexed, cloneable key-value map. You populate it at route definition time:

```rust
let metadata = RouteMetadata::new();
metadata.insert(PublicEndpoint);       // a unit struct
metadata.insert(ApiVersion("v2"));     // a newtype
metadata.insert(RateLimit { max: 100 }); // a config struct
```

And you retrieve it in your filter:

```rust
fn catch(&self, error: &HttpError, ctx: &FilterContext) -> Result<FrameworkResponse, HttpError> {
    if let Some(public) = ctx.route_metadata().get::<PublicEndpoint>() {
        // This route is public — return a friendlier error page
    }
    Err(error.clone())
}
```

The metadata map is the bridge between route declaration and error recovery. Attach whatever typed context helps your error handlers make decisions — rate limit tiers, deprecation state, ownership domains.

## How `ExceptionFilterSet` chains filters

At `exception_filter.rs:48`, `ExceptionFilterSet` is a simple newtype over `Vec<Arc<dyn ExceptionFilter>>`. Its `catch` method (line 73) is where the chain logic lives:

```rust
pub(crate) fn catch(
    &self,
    error: &HttpError,
    context: &FilterContext,
) -> Option<Result<FrameworkResponse, HttpError>> {
    for filter in &self.filters {
        let result = filter.catch(error, context);
        if let Ok(ref _resp) = result {
            return Some(result);  // handled — stop iterating
        }
        if result.is_err() {
            return Some(result);  // filter errored — bubble that up
        }
    }
    None  // no filter matched; caller falls through to default
}
```

Two things to notice. First, iteration is FIFO — filters registered first run first. Second, a filter that returns `Err(...)` _does_ stop the chain, but with an error rather than a response. This means a filter that itself fails (e.g. an internal bug producing `Err(error)`) will propagate immediately rather than silently skipping to the next filter. The distinction between "I didn't handle this" (returning `None`, via the caller) and "I tried and broke" (returning `Some(Err(...))`) is explicit.

## Scope precedence: route → controller → global

The dispatch order is wired at `pipeline.rs:230–251`:

```rust
Err(error) => {
    let filter_ctx = crate::FilterContext::new(route.metadata().clone());
    // Route-level filters (includes controller filters, most specific first)
    if let Some(result) = route
        .pipeline()
        .exception_filters
        .catch(&error, &filter_ctx)
    {
        return result;
    }
    // Global-level filters
    if let Some(result) = application
        .pipeline()
        .exception_filters
        .catch(&error, &filter_ctx)
    {
        return result;
    }
    Err(error)  // fallback: raw error propagates
}
```

There appear to be only two levels — route and global — but the third (controller) is merged in during compilation. At `route.rs:404–405`, when `ControllerDefinition::compile_routes()` builds a `CompiledRoute`, it clones the controller's pipeline and appends the route's pipeline on top:

```rust
let mut pipeline = self.pipeline.clone();   // controller-level
pipeline.append(&route.pipeline);           // route-level merged in
```

Because `PipelineComponents::append` (pipeline.rs:177) adds the other set's filters _after_ its own, the resulting order is: controller filters first, then route filters, then global filters — exactly the "most specific first" contract.

## Fallback to default — raw error through the onion

If no filter at any scope catches the error, `execute` returns `Err(error)` (line 250). This `HttpError` unwinds back through the middleware onion — each middleware's `handle` returns `Err(...)`, which the caller's `?` propagates upward. Eventually it reaches the server adapter, which calls `IntoFrameworkResponse::into_framework_response` on it (`error.rs:87`), producing a minimal JSON body with `{ "status": 500, "code": "...", "message": "..." }`.

The bottom line: if you register no exception filters, every error hits the adapter as-is. Add a global filter and you intercept everything. Add route-level filters and you override per-endpoint.

## Why filter errors are different from guard denials

Guards and exception filters both produce `HttpError`, but they enter the pipeline at different points. Guards run _before_ interceptors and handlers, at `pipeline.rs:273–296`. When a guard denies, it returns `HttpError::forbidden(...)` directly — the error never reaches an exception filter because the guard's denial _is_ the error being returned from `run_guards`.

Exception filters, by contrast, catch errors thrown from anywhere in the pipeline downstream of the middleware layer: extraction failures, interceptor rejections, handler panics, serialization bugs. The error bubbles up through the middleware stack via `?` propagation, then hits the `Err` arm in `execute` (line 232) where filters get their turn.

This means you can use exception filters to add error handling _to existing middleware_. A logging middleware that wraps `next.run(context).await` with `?` will have its errors caught by filters just like handler errors would. Guards, however, short-circuit before the handler ever runs — they're an access-control decision, not a recoverable application error.

## Concrete example: `NotFoundFilter`

Here's a filter that catches any 404 — whether from a missing route or a handler returning `HttpError::not_found(...)` — and replaces the default JSON with a structured error:

```rust
use ironic_http::{ExceptionFilter, FilterContext, FrameworkResponse, HttpError, Json};

struct NotFoundFilter;

impl ExceptionFilter for NotFoundFilter {
    fn catch(
        &self,
        error: &HttpError,
        _ctx: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError> {
        if error.status().is_not_found() {
            let body = serde_json::json!({
                "error": {
                    "code": error.code(),
                    "message": error.message(),
                    "type": "https://api.example.com/errors/not-found"
                }
            });
            Ok(FrameworkResponse::json(
                ironic_http::HttpStatus::NOT_FOUND,
                body,
            ))
        } else {
            Err(error.clone()) // pass through — let another filter handle it
        }
    }
}
```

The filter inspects `error.status()` rather than `error.code()` so it catches _any_ 404 regardless of the source error code. When the status doesn't match, it returns `Err(error.clone())` — the error moves on to the next filter in the chain unchanged.

## Registering filters at all three levels

**Route level** — use `.exception_filter(...)` on the route definition:

```rust
.route(
    RouteDefinition::new(HttpMethod::GET, "/users/:id", "show", handler_fn(show_user))?
        .exception_filter(Arc::new(NotFoundFilter))
)
```

**Controller level** — use `#[exception(...)]` on the struct, applied to all routes:

```rust
#[controller("/users")]
#[exception(NotFoundFilter)]
#[derive(Injectable)]
struct UserController;
```

Every route in the controller inherits this filter. During compilation, controller filters are merged into each route's pipeline as the first layer.

**Global level** (`route.rs:633`):

```rust
CompiledHttpApplication::new(container, routes)
    .exception_filter(Arc::new(NotFoundFilter))
```

Global filters run last, after all route and controller filters have had their chance. This is where you put catch-all handlers — convert `InternalServerError` into a safe public message, or log every unhandled error to your observability pipeline.

---

The exception filter system is one of the smallest subsystems in Ironic — under 90 lines for the core trait and set, plus 20 lines of dispatch in the pipeline — but it gives you full control over how errors surface to clients. Define once at the right scope, compose with metadata-driven decisions, and let the chain do the rest.
