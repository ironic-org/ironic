---
title: "PipelineComponents — how middleware, guards, and interceptors span three scopes"
description: "A deep dive into the PipelineComponents struct, the ExecutionState interleaving strategy, and how global, controller, and route-level components compose without extra generics."
date: "2026-07-15"
author: "Ironic Team"
---

# PipelineComponents — how middleware, guards, and interceptors span three scopes

Ironic's request pipeline has an unusual property: middleware, guards, and interceptors can be registered at three different scopes — global application, controller, and individual route — and they all interleave in a predictable order. A global middleware runs before a controller middleware, which runs before a route middleware. The same holds for guards and interceptors.

Most frameworks implement this with three separate types, three separate vecs, and three separate iteration strategies. Ironic does it with one type and one index arithmetic pattern.

---

## One struct, four collections

Open `crates/ironic-http/src/pipeline.rs:117`:

```rust
pub struct PipelineComponents {
    pub(crate) middleware: Vec<Arc<dyn Middleware>>,
    pub(crate) guards: Vec<Arc<dyn Guard>>,
    pub(crate) interceptors: Vec<Arc<dyn Interceptor>>,
    pub(crate) exception_filters: ExceptionFilterSet,
}
```

That is the entire type. Four trait-object collections, each erased behind `Arc<dyn ...>`. There is no generic parameter, no lifetime, no scope discriminator. The same `PipelineComponents` represents global middleware in `CompiledHttpApplication`, controller-level middleware in `ControllerDefinition`, and route-level middleware in `RouteDefinition`.

The build methods are equally uniform (`pipeline.rs:149`):

```rust
pub fn middleware(mut self, middleware: impl Middleware) -> Self {
    self.middleware.push(Arc::new(middleware));
    self
}
```

`guard()` and `interceptor()` follow the exact same pattern. Each takes ownership of `self`, wraps the concrete type in `Arc`, pushes it, and returns. The builder pattern forces immutability: you cannot add middleware to an existing `PipelineComponents` after construction. You can only chain builder calls.

---

## ExecutionState — two slices, one merged view

When a request arrives, the pipeline needs to see global and route-local components as a single flat sequence. The `ExecutionState` at `pipeline.rs:186` is the bridge:

```rust
struct ExecutionState<'a> {
    application: &'a CompiledHttpApplication,
    route: &'a CompiledRoute,
}
```

It bundles a reference to the application (which holds the global `PipelineComponents`) and a reference to the matched route (which holds the merged controller+route `PipelineComponents`). The global `PipelineComponents` is separate because the application builder owns it; the route's `PipelineComponents` has already absorbed the controller's components during compilation.

---

## Index arithmetic: one function, two sources

The core interleaving logic lives in three lookup functions. Here is `middleware_at` (`pipeline.rs:339`):

```rust
fn middleware_at(state: &ExecutionState, index: usize) -> Option<&Arc<dyn Middleware>> {
    let global = &state.application.pipeline().middleware;
    global.get(index)
        .or_else(|| state.route.pipeline().middleware.get(index - global.len()))
}
```

If `index` is less than the global vec length, it's a global middleware. Otherwise, subtract the global count — the adjusted index is a route-local (controller+route) middleware. The iteration starts at index 0 and increments. The global vec is consumed first, then the route-local vec. Two vecs, one index space.

`guard_at()` and `interceptor_at()` use the same pattern with different fields. `guard_at()` works on `state.application.pipeline().guards` and `state.route.pipeline().guards`. `interceptor_at()` works on `state.application.pipeline().interceptors` and `state.route.pipeline().interceptors`. Three lookup functions, one arithmetic strategy.

For guards, there is an additional nuance: guards are not chained recursively like middleware. Instead, `run_guards` iterates all guards sequentially in a loop (`pipeline.rs:273`). If any guard returns `Deny`, the pipeline stops. The guard count function sums both vec lengths: `state.application.pipeline().guards.len() + state.route.pipeline().guards.len()`.

---

## append() — merging controller into route

Controller-level components are not a separate slice at runtime. During route compilation (`route.rs:404`), the controller's `PipelineComponents` is cloned and merged into each route:

```rust
let mut pipeline = self.pipeline.clone();  // self = ControllerDefinition
pipeline.append(&route.pipeline);
```

The `append()` method (`pipeline.rs:177`) extends each vec:

```rust
pub(crate) fn append(&mut self, other: &Self) {
    self.middleware.extend(other.middleware.iter().cloned());
    self.guards.extend(other.guards.iter().cloned());
    self.interceptors.extend(other.interceptors.iter().cloned());
    self.exception_filters.append(&mut other.exception_filters.clone());
}
```

Controller middleware sits before route middleware because `append` extends the controller's vec with the route's entries. When `middleware_at` later scans the route-local vec, controller middleware appears at lower indices than route middleware. The global vec, being a separate source, naturally comes first.

---

## Three registration paths, one type

The user-facing API has three entry points, but they all delegate to the same `PipelineComponents::middleware()` method:

| Scope | Builder | Method |
|-------|---------|--------|
| Global | `CompiledHttpApplication` | `.middleware(impl Middleware)` — `route.rs:605` |
| Controller | `ControllerDefinition` | `.middleware(impl Middleware)` — `route.rs:323` |
| Route | `RouteDefinition` | `.middleware(impl Middleware)` — `route.rs:168` |

Each builder holds its own `PipelineComponents` field. The global builder's field becomes `application.pipeline()` at runtime. The controller's field gets cloned and merged into each route during compilation. The route's field gets absorbed into the controller's during `append()`. Guards, interceptors, and exception filters follow the same three-path pattern.

---

## Why no extra generics

The key design insight is that `PipelineComponents` does not parameterize over scope. It doesn't need a `Scope` enum, a `where Scope: PipelineScope` bound, or a phantom type. It just holds vecs of trait objects. The scope is determined entirely by _which struct owns the `PipelineComponents` instance_. A `CompiledHttpApplication` owns the global one. A `CompiledRoute` owns the merged controller+route one. The interleaving logic at runtime treats both as opaque slices and stitches them together via index arithmetic.

This means you can write middleware once and register it anywhere — globally, per controller, or per route — without type-level changes. The same `impl Middleware` type works with all three builder methods because all three accept `impl Middleware` via the same trait bound and push it into the same `Vec<Arc<dyn Middleware>>`.
