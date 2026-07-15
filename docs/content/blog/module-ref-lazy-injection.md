---
title: "ModuleRef — lazy container injection via Arc<OnceLock<Container>>"
description: "How Ironic solves the bootstrapping chicken-and-egg problem with ModuleRef: a lazily-populated container handle that allows services to resolve dependencies at runtime long after build() completes."
date: "2026-07-15"
author: "Ironic Team"
---

# ModuleRef — lazy container injection via `Arc<OnceLock<Container>>`

Dependency injection containers face a classic chicken-and-egg problem at startup. Services need the container to resolve their own dependencies, but the container doesn't exist until all services are registered. Most frameworks sidestep this by requiring every dependency to be declared statically in the constructor. But what happens when you have a background task scheduler that spawns dynamic work, or a plugin system that resolves services by name, or a request handler that needs to call into a provider whose type isn't known until runtime?

Ironic's answer is `ModuleRef` — a lazily-populated handle to the DI container that can be injected into any service and used to resolve dependencies long after `build()` returns.

---

## The problem: nothing exists until `build()`

In Ironic, a `ContainerBuilder` collects `ProviderDefinition` registrations and, when `build()` is called, compiles them into a `Container` with a fully initialized provider graph. Until `build()` completes, there is no container. But some services need container access to do their jobs — a job scheduler might need to resolve `Arc<dyn JobHandler>` dynamically, or a health-check aggregator might need to enumerate every registered `Arc<dyn HealthCheck>`.

The conventional DI answer is: don't do that. Declare all your dependencies in the constructor and let the container wire them. But that doesn't work when the set of dependencies is dynamic or when resolution must happen outside the construction phase, such as in a background loop or a custom scope.

The `Arc<Container>` pattern (register the container into itself) is a well-known workaround, but it has a fatal flaw: `ContainerBuilder` doesn't *have* a `Container` to register during the builder phase. The container is the output of `build()`, not an input.

---

## The solution: `ModuleRef` is created before `build()`

`ModuleRef` is a small wrapper around `Arc<OnceLock<Container>>`:

```rust
pub struct ModuleRef {
    container: std::sync::Arc<OnceLock<Container>>,
}
```

`OnceLock` is Rust's standard library primitive for one-time initialization — it starts empty, can be set exactly once, and safely blocks or returns `None` depending on how you read it. In `ModuleRef`, the `OnceLock` starts empty and is populated *after* `build()` succeeds.

The lifecycle has three phases:

**Phase 1 — Registration (before build)**: The `ModuleRef` is created while the builder is still collecting registrations. At this point, `container` is empty. The `ModuleRef` is registered as a value-based provider:

```rust
let module_ref = std::sync::Arc::new(ModuleRef::new());
let module_ref_provider = ProviderDefinition::value(module_ref.clone());
```

Because it's a value provider (not a factory), the DI container simply hands out the same `Arc<ModuleRef>` instance to every service that depends on it. No construction logic runs; the `ModuleRef` already exists.

**Phase 2 — Population (during build)**: After the container graph is compiled and the HTTP application is constructed, the framework calls `set_container()`:

```rust
module_ref.set_container(container.clone());
```

This one call writes the fully-constructed `Container` into the `OnceLock`. From this point forward, any service holding an `Arc<ModuleRef>` can resolve providers through it.

**Phase 3 — Resolution (at runtime)**: Services injected with `Arc<ModuleRef>` call `module_ref.resolve::<T>()` to lazily resolve any registered provider. The `resolve` method checks whether the `OnceLock` has been populated and forwards the call to the underlying `Container::resolve`.

---

## What happens if you call `resolve()` too early?

The `resolve()` method handles the uninitialized case explicitly:

```rust
pub async fn resolve<T: Send + Sync + 'static>(
    &self,
) -> Result<std::sync::Arc<T>, ironic_di::ResolveError> {
    self.container
        .get()
        .ok_or_else(|| {
            let key = ironic_di::ProviderKey::of::<T>();
            ironic_di::ResolveError::MissingProvider { key, path: Vec::new() }
        })?
        .resolve::<T>()
        .await
}
```

If `container.get()` returns `None` (the `OnceLock` hasn't been set yet), the method returns a `MissingProvider` error rather than panicking. This is a deliberate choice — a panic from a `OnceLock` that hasn't been initialized would crash the application with no context. A `ResolveError` is a recoverable error type that can be logged, reported through a health endpoint, or propagated as an HTTP 500. It tells you *what* went wrong (the container doesn't have this provider) and provides a dependency chain path for debugging.

---

## A concrete example

Consider a background task scheduler that spawns jobs dynamically:

```rust
struct TaskScheduler {
    module_ref: Arc<ModuleRef>,
}

impl TaskScheduler {
    async fn run(&self) {
        loop {
            let task_type = self.dequeue_task().await;
            // Resolve the handler dynamically — not known at construction time
            let handler: Arc<dyn TaskHandler> = self.module_ref
                .resolve::<DynamicTaskHandler>()
                .await
                .unwrap();
            handler.execute(task_type).await;
        }
    }
}
```

The scheduler doesn't know which specific `TaskHandler` implementation it needs when it's constructed — that depends on the task type dequeued at runtime. By holding `Arc<ModuleRef>`, it can resolve any registered provider at any point in its lifecycle, deferring the DI resolution from construction time to invocation time.

For cases where the provider may or may not be registered, `resolve_optional()` returns `Ok(None)` instead of an error when the provider is absent, making optional dependency patterns ergonomic.

---

## Why `OnceLock` instead of `Mutex<Option<Container>>`?

`OnceLock` has two properties that make it ideal for this use case. First, it enforces write-once semantics at the type level — the framework *cannot* accidentally overwrite the container after initialization, which would be a catastrophic correctness bug. Second, `OnceLock::get()` on a populated lock is a single relaxed atomic load — no contention, no lock acquisition, just a pointer read. For a hot path like dependency resolution, avoiding a mutex acquisition on every `resolve()` call matters.

The `Arc` wrapper provides shared ownership with zero per-clone overhead beyond reference-counting — every service that depends on `ModuleRef` gets a handle to the exact same `OnceLock`, and when the container is set, every handle sees it immediately through the shared state.
