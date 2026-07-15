---
title: "The Pipe System — validation and transformation chains for erased values"
description: "How Ironic's ParameterPipe trait chains type-erased validation and transformation steps — from the erased Box<dyn Any> through SyncPipe, built-in parsers, garde integration, and global/controller/route ordering."
date: "2026-07-15"
author: "Ironic Team"
---

# The Pipe System — validation and transformation chains for erased values

By the time extraction finishes in Ironic, every handler argument is an `ExtractedValue` — `Box<dyn Any + Send>`. There's no `u64`, no `bool`, no `Uuid`. Just an opaque pointer behind a vtable. Yet before the value reaches your handler, it must be parsed, validated, and range-checked — against concrete types the framework itself never sees.

The Pipe System solves this: a chain of type-aware transformations on type-erased values, where each link downcasts to recover the concrete type, applies its logic, and re-erases the result for the next link.

## The trait: one contract for every transformation

Every pipe implements `ParameterPipe` (`pipeline.rs:56`):

```rust
pub trait ParameterPipe: Send + Sync + 'static {
    fn transform<'a>(
        &'a self,
        value: ExtractedValue,
        context: &'a mut RequestContext,
    ) -> PipeFuture<'a>;

    fn description(&self) -> &'static str;
}
```

`PipeFuture<'a>` is `Pin<Box<dyn Future<Output = Result<ExtractedValue, HttpError>> + Send + 'a>>`. The pipe receives an erased value and mutable access to `RequestContext`, returning either a transformed erased value or an `HttpError`.

The contract is deliberately narrow. A pipe doesn't know which parameter or route it's validating — it receives one value and produces one value, or fails. This makes pipes composable.

## ExtractedValue: the erased type at rest

`ExtractedValue` (`extract.rs:8`) is the lowest common denominator:

```rust
pub type ExtractedValue = Box<dyn Any + Send>;
```

After extraction, a path parameter like `"42"` is `Box<dyn Any + Send>` containing a `String`. A JSON body field is the same. Every parameter lands in this uniform container before the pipe chain runs.

The trade-off: every pipe must open the box with a typed downcast. If the type doesn't match, the pipe returns an error. The framework guarantees correctness at compile time, but the runtime still performs the check.

## SyncPipe: bridging typed closures to the erased trait

Writing a full `ParameterPipe` impl for every rule is tedious. `SyncPipe<T, U, F>` (`pipeline.rs:68`) wraps any synchronous `Fn(T) -> Result<U, HttpError>` into the trait:

```rust
struct SyncPipe<T, U, F> {
    transform: F,
    marker: PhantomData<fn(T) -> U>,
}
```

Its `transform` unpacks the value via `downcast::<T>()`, applies the closure, and repacks. On a type mismatch it returns `RF_HTTP_PIPE_TYPE_MISMATCH`. On success the result is wrapped in `Box::pin(async move { result })` to satisfy the trait's future requirement.

The higher-order `pipe_fn()` function (`pipeline.rs:104`) constructs a `SyncPipe` behind an `Arc<dyn ParameterPipe>`:

```rust
pub fn pipe_fn<T, U, F>(transform: F) -> Arc<dyn ParameterPipe>
where
    T: Send + Sync + 'static,
    U: Send + Sync + 'static,
    F: Fn(T) -> Result<U, HttpError> + Send + Sync + 'static,
```

This lets you build ad-hoc pipes from any closure:

```rust
let pipe = pipe_fn::<String, String, _>(|s| Ok(s.to_lowercase()));
```

No struct definition needed. No trait impl needed. Just a closure.

## Built-in pipes

Ironic ships with pre-built pipes in `pipes.rs`:

- **`ParseIntPipe`** — `String` → `i64`. Returns 400 (`RF_PARSE_INT_FAILED`).
- **`ParseFloatPipe`** — `String` → `f64` (`RF_PARSE_FLOAT_FAILED`).
- **`ParseBoolPipe`** — `String` → `bool`. Accepts `"true"`, `"false"`, `"1"`, `"0"` (case-insensitive).
- **`ParseUUIDPipe`** — `String` → `uuid::Uuid`. Gated behind the `uuid` feature.
- **`ValidationPipe`** — Gated behind the `validation` feature. A marker: the actual validation is driven by `garde` derive macros, inserted at compile time. The pipe passes the value through unchanged.

Each has a convenience constructor (`parse_int()`, `parse_float()`, etc.) returning `Arc<dyn ParameterPipe>`.

## How pipes chain

A `ParameterDefinition` (`route.rs:17`) pairs an extractor with an ordered `Vec<Arc<dyn ParameterPipe>>`. The execution loop in `CompiledRoute::invoke_handler` (`route.rs:520`):

```rust
let mut value = parameter.extractor.extract(context).await?;
for pipe in &parameter.pipes {
    value = pipe.transform(value, context).await?;
}
arguments.push(value);
```

The output of pipe `N` becomes the input of pipe `N+1`. If any pipe returns `Err`, the `?` operator immediately stops the loop and the handler is never called.

## Global → controller → route: the ordering rule

Pipes compose across three scopes:

1. **Global** — registered on `CompiledHttpApplication` via `.pipe(&pipe)` (`route.rs:641`). Inserted at index 0, so they run first.
2. **Controller** — registered on `ControllerDefinition` via `.pipe(pipe)` (`route.rs:358`). Prepended before route pipes during compilation (`route.rs:411-414`).
3. **Route** — registered per-parameter via `.parameter_with_pipe(extractor, pipe)` (`route.rs:140`) or `.parameter_with_pipes()`.

Final order: **global → controller → route**. The test at `pipeline.rs:879` confirms this with labeled pipes, asserting the sequence `["global-pipe", "controller-pipe", "route-pipe"]`.

## garde integration

With the `validation` feature enabled, `ValidationPipe` (`pipes.rs:175`) acts as a marker. Your DTOs use garde's derive:

```rust
#[derive(garde::Validate)]
struct CreateUser {
    #[garde(length(min = 1))]
    name: String,
    #[garde(email)]
    email: String,
}
```

The route-generation macros detect the `Validate` implementation and insert validation calls between extraction and the handler. The `ValidationPipe` struct itself doesn't invoke garde — it passes the value through unchanged. The real validation work is done by the macro-generated code that reads the `garde::Validate` impl. Separating the marker from the logic means a single `ValidationPipe` instance can validate any type without being generic over it.

## What the caller sees when a pipe fails

When a pipe returns an `HttpError`, the chain stops immediately. The handler never executes. The error unwinds through interceptors and middleware (each of which gets a chance to observe it via their after-hooks), then reaches the exception filter layer. Route-level exception filters get first crack, then global ones. If none handle it, the caller receives the raw `HttpError`.

A test at `pipeline.rs:760` verifies this explicitly: when a recording pipe is set to fail, the test confirms that `"pipe"` appears in the event log but `"handler"` does not. The chain stopped exactly at the failing pipe.

## Concrete example: user ID parameter

Here's a realistic chain for a user ID path parameter:

```rust
use ironic_http::{parse_int, pipe_fn, HttpError, HttpStatus};

fn range_pipe(min: i64, max: i64) -> Arc<dyn ParameterPipe> {
    pipe_fn::<i64, i64, _>(move |value| {
        if value < min || value > max {
            Err(HttpError::new(
                HttpStatus::BAD_REQUEST,
                "RANGE_ERROR",
                format!("Value must be between {min} and {max}, got {value}"),
            ))
        } else {
            Ok(value)
        }
    })
}

let route = RouteDefinition::new(
    HttpMethod::GET,
    "/users/:id",
    "get_user",
    handler_fn(|_controller: Arc<Controller>, mut args| async move {
        let user_id: i64 = args.take(0)?;
        Ok(json_response(&get_user(user_id)))
    }),
)?
.parameter_with_pipes(
    PathParameter::new("id"),
    [parse_int(), range_pipe(1, 100)],
);
```

The flow: the path extractor pulls `"42"` as a `String` wrapped in `ExtractedValue`. `ParseIntPipe` downcasts to `String`, parses to `i64`, and re-erases as `Box<dyn Any + Send>`. `RangePipe` downcasts to `i64`, checks the bounds, and passes it through. If the value is `"0"` or `"101"`, or if the path contains `"abc"`, the handler never runs — the caller gets a 400 with a specific error code and message.

This is the Pipe System in microcosm: erased values, typed transformations, sequential chaining, and fail-fast semantics — all without a single generic on the `ParameterPipe` trait itself.
