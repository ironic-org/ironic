---
title: "Type-erased yet type-safe — how Ironic dispatches handlers without reflection"
description: "A deep dive into Ironic's zero-reflection handler dispatch: Box<dyn Any>, proc-macro binding, and why type mismatches are structurally impossible."
date: "2026-07-15"
author: "Ironic Team"
---

# Type-erased yet type-safe — how Ironic dispatches handlers without reflection

Every Rust HTTP framework must solve the same collection problem: how do you store a set of route handlers in a `Vec` when each handler has a different signature? An Axum handler `async fn get(id: u64) -> Json<User>` is a completely different type from `async fn list(page: u32) -> Json<Vec<User>>`. Rust has no runtime reflection, no universal `dyn Callable`, and heterogeneous collections are famously awkward in a monomorphized language. Ironic solves this with an approach that is entirely type-erased at the storage layer yet provably type-safe at the call site — without a single `Reflect` derive or `bevy_reflect` in sight.

## The layer cake

Ironic's dispatch pipeline runs through three layers of abstraction, each narrower than the last:

1. **Compiled routes** (`CompiledRoute`, `route.rs:456`) store the handler as `Arc<dyn ErasedHandler>` — a trait object in a `Vec<CompiledRoute>` inside `CompiledHttpApplication`. The router matches a request to a route, but it knows nothing about the concrete handler type.

2. **The erased handler** (`ErasedHandler`, `handler.rs:51`) is the trait that every route handler gets erased into. It has a single method:

```rust
fn call(&self, controller: ProviderValue, arguments: HandlerArguments) -> HandlerFuture;
```

The important thing is what this trait does *not* have: no generics, no associated types, no mention of any concrete parameter types. It is a fully type-erased interface.

3. **The handler function** (`handler_fn`, `handler.rs:84`) is the bridge. It takes a closure with a concrete signature — `Fn(Arc<C>, HandlerArguments) -> Fut` — and wraps it in `Arc<dyn ErasedHandler>`. The concrete types are captured in the closure body and become invisible to the caller.

## How `HandlerArguments` carries erased values

Between extraction and invocation sits `HandlerArguments` (`handler.rs:14`), a thin wrapper around `Vec<Option<ExtractedValue>>`. And `ExtractedValue` is simply:

```rust
pub type ExtractedValue = Box<dyn Any + Send>;
```

(`extract.rs:8`)

Each parameter the framework extracts — a path segment, a query string, a JSON body — gets boxed into a `dyn Any`. This is the single allocation we permit per parameter. A handler with three parameters pays three allocations; a handler with zero pays none.

Why `Option` in the vec? Because `take::<T>(index)` (`handler.rs:30`) consumes the argument by calling `Option::take`. After downcasting, the slot is `None` — preventing accidental reuse and giving the caller ownership without cloning.

## Extraction: from the wire to a box

Consider `PathParameter<T>` (`extract.rs:25`). It stores nothing more than a `&'static str` name:

```rust
pub struct PathParameter<T> {
    name: &'static str,
    marker: PhantomData<fn() -> T>,
}
```

When `extract()` fires (`extract.rs:46`), it reaches into Axum's request extensions, pulls the raw path parameter string, calls `raw.parse::<T>()`, and boxes the result:

```rust
Ok(Box::new(value) as ExtractedValue)
```

The `Box<u64>` is immediately upcast to `Box<dyn Any + Send>`. The concrete type `u64` vanishes. The route table never sees it.

`JsonBody<T>`, `QueryParameters<T>`, and `HeaderParameter<T>` follow the same pattern: deserialize from the request in a strongly typed context, then immediately erase the type into a `dyn Any` box. The extractor knows the type; the rest of the framework does not need to.

## Invocation: downcasting back to the concrete type

Look at how the proc macro generates the handler wrapper at `routes.rs:174-183`:

```rust
::ironic::handler_fn(
    |controller: ::std::sync::Arc<#self_ty>, mut arguments| async move {
        let id = arguments.take::<u64>(0)?;
        let body = arguments.take::<JsonBody<User>>(1)?;
        controller.#method_name(id, body).await
    },
)
```

For each handler parameter, the macro emits a `take::<T>(index)` call (`routes.rs:154`). This is the crucial moment: `take` calls `value.downcast::<T>()` on the erased `Box<dyn Any>` (`handler.rs:41`). If the downcast succeeds, the handler receives a perfectly typed value. If it fails — wrong type at the wrong index — it returns an `HttpError` with the code `RF_HTTP_HANDLER_TYPE_MISMATCH`.

## Why type mismatches are structurally impossible

The downcast can't fail in practice. Why? Because the *same proc macro* that emits the downcast calls also emits the extractor chain:

```rust
// routes.rs:167 — generated extractor calls
.parameter(::ironic::PathParameter::<u64>::new("id"))
.parameter(::ironic::JsonBody::<User>::new())
```

The extractors at index 0 and 1 produce `Box<u64>` and `Box<User>` respectively. The downcasts at index 0 and 1 demand `u64` and `User` respectively. These are generated from the same function signature in the same macro expansion. They cannot disagree. No runtime check is needed — the proc macro is the proof.

If you change the handler signature from `id: u64` to `id: String`, the macro regenerates *both* the extractor and the downcast. The types stay locked together. A mismatch would require a bug in the macro itself, not in user code.

## The runtime cost

The cost per extracted parameter is:

1. **One heap allocation**: `Box::new(value)` during extraction.
2. **One vtable lookup + data pointer read**: `downcast::<T>()` during invocation.

Both are measured in single-digit nanoseconds on modern hardware. There is no hashing, no string comparison, no `TypeId` map lookup — the index into the `Vec` is baked into the generated code at compile time. The `Vec` is pre-allocated to exactly `self.parameters.len()` (`route.rs:525`), so there are no reallocations either.

For comparison, a typical framework using `Box<dyn Any>` for middleware state or request extensions pays these same costs per-request anyway. Ironic simply applies the same technique to handler parameters, with the additional guarantee that the static types at construction and consumption are derived from one source.

## A concrete trace: `GET /users/42`

Here is the full lifecycle of a single parameter through the system:

---

**1. Route match.** The compiled application's router matches `GET /users/:id` and selects the appropriate `CompiledRoute`. The route holds `Arc<dyn ErasedHandler>` and one `ParameterDefinition` wrapping `PathParameter::<u64>::new("id")`.

**2. Extraction.** `invoke_handler` (`route.rs:520`) iterates parameters. `PathParameter::<u64>::extract()` calls Axum's `request.path_parameter("id")`, gets `"42"`, parses it to `42u64`, and returns `Ok(Box::new(42u64) as ExtractedValue)`. The type `u64` is gone.

**3. Arguments.** The `ExtractedValue` is pushed into a `Vec`, wrapped in `HandlerArguments::new()`. The vector now holds `[Some(Box<42u64> as Box<dyn Any>)]`.

**4. Dispatch.** `self.handler.call(controller, arguments)` invokes the erased handler. Inside the `HandlerFn` impl (`handler.rs:68`), the controller is downcast from `ProviderValue`, then the closure generated by the proc macro runs.

**5. Downcast.** `arguments.take::<u64>(0)` takes the boxed value, downcasts it, and returns `Ok(42u64)`. The handler function receives `42u64` — exactly the type it expects.

**6. Response.** The handler returns `Json<User>`, which gets converted to `Response` via `into_framework_response()`, and the bytes hit the wire.

---

At no point does the router or any middleware know that the handler takes a `u64`. The `CompiledRoute` stores `Arc<dyn ErasedHandler>` and `Vec<ParameterDefinition>` — both type-erased. The concrete types live exclusively inside the proc-macro-generated closure and the extractor implementations, both of which are monomorphized at compile time. This is type erasure as an architectural boundary, not as a loss of information: the types are temporarily hidden, stored as opaque boxes, and restored precisely where they are needed. No reflection, no unsafe, no magic — just the `Any` trait and a proc macro that keeps both sides of the bridge in lockstep.
