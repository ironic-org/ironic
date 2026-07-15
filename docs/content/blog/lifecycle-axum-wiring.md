---
title: "Lifecycle hooks under the hood — how OnModuleInit connects to axum::serve"
description: "A full trace of how Ironic's lifecycle system runs outside Axum's service stack — from compile_module_graph through bootstrap, serve, and shutdown."
date: "2026-07-15"
author: "Ironic Team"
---

# Lifecycle hooks under the hood — how OnModuleInit connects to axum::serve

The lifecycle hooks (`OnModuleInit`, `OnApplicationBootstrap`, `OnModuleDestroy`, `OnApplicationShutdown`) seem like they should be wired into Axum's middleware stack. They're not. They run in a completely separate orchestration layer that wraps the Axum server, not threads through it.

Let me trace the full path.

---

## The three phases

There are exactly three phases, each with a clear entry point in `FrameworkApplication`:

| Phase | When | Where |
|-------|------|-------|
| **Init** | Before `axum::serve` | `FrameworkApplicationBuilder::build()` |
| **Serve** | Inside `axum::serve` | `platform.listen(address, shutdown)` |
| **Shutdown** | After `axum::serve` returns | `shutdown_application()` and `destroy_modules()` |

The bridge between them is the `initialized` vector — a plain `Vec<InitializedLifecycle>` created during build, carried through the `FrameworkApplication` struct, and consumed during shutdown.

---

## Phase 1 — Init (before the server binds)

`FrameworkApplicationBuilder::build()` at `application.rs:181` is where everything comes together. The order is strict:

```rust
pub async fn build(self) -> Result<FrameworkApplication<A::Application>, ApplicationError> {
    let module_ref = Arc::new(ModuleRef::new());
    let graph = compile_module_graph(root)?;                               // ①
    let http = build_http_application_with_extra_providers(...)?;          // ②
    let container = http.container().clone();

    initialize_eager_providers(&graph, &container).await?;                // ③
    let mut initialized = Vec::new();
    initialize_lifecycle(&graph, &container, &mut initialized).await?;    // ④
    bootstrap_application(&initialized).await?;                            // ⑤

    let platform = self.adapter.build(Arc::new(http))?;                   // ⑥

    Ok(FrameworkApplication { graph, container, platform, initialized })
}
```

### ① `compile_module_graph()` — compute the order

The module graph compiler does a topological sort of imports. The result is two lists:

```rust
initialization_order: Vec<ModuleId>   // parents before children
shutdown_order: Vec<ModuleId>         // reverse of init order (children before parents)
```

A `ModuleDefinition` with `imports = [DatabaseModule]` will be ordered AFTER `DatabaseModule` in init. During shutdown, it will be destroyed FIRST, reversing that order.

### ② `build_http_application_with_extra_providers()` — build the container

This walks every module, registers every provider and controller into a `ContainerBuilder`, applies overrides, resolves the final `Container`, and compiles routes. At this point the DI container is alive but no provider has been constructed yet — they're all lazy.

### ③ `initialize_eager_providers()` — force construction now

For any provider marked `#[injectable(eager)]`, we call `container.resolve_key()` immediately. This triggers the singleton's `OnceCell` initialization, which calls the factory closure. Database pools, external service connections — anything that should fail fast at startup rather than at 3 AM on the first request:

```rust
async fn initialize_eager_providers(graph, container) -> Result {
    for module_id in graph.initialization_order() {
        for provider in module.providers().iter().filter(|p| p.is_eager()) {
            container.resolve_key(provider.key()).await?;  // ← forces construction
        }
    }
}
```

If an eager provider fails, the error is returned immediately. No further providers are initialized. Already-initialized providers get `OnModuleDestroy` called in reverse before the error propagates — no half-initialized state.

### ④ `initialize_lifecycle()` — call OnModuleInit

This is where `OnModuleInit` actually fires. For each module in `initialization_order`, for each lifecycle definition in that module:

```rust
async fn initialize_lifecycle(graph, container, initialized) -> Result {
    for module_id in graph.initialization_order() {
        for definition in module.lifecycle() {
            // Resolve the provider from the container
            let provider = container.resolve_key(definition.key()).await?;

            // Store for later (shutdown needs this)
            initialized.push(InitializedLifecycle {
                definition: definition.clone(),
                provider: Arc::clone(&provider),
            });

            // Call OnModuleInit
            if let Some(callback) = &definition.module_init {
                callback(provider).await?;
            }
        }
    }
}
```

The crucial line is `initialized.push(...)`. The `InitializedLifecycle` struct is the permanent record of what was called and with what provider value. It's what makes shutdown deterministic — when the server stops, we walk this vec in reverse and call the cleanup callbacks with the same provider values.

### ⑤ `bootstrap_application()` — call OnApplicationBootstrap

After EVERY module has finished init, the bootstrap phase runs. This is a second pass over the initialized vec, this time calling `application_bootstrap` callbacks:

```rust
async fn bootstrap_application(initialized) -> Result {
    for lifecycle in initialized {
        if let Some(callback) = &lifecycle.definition.application_bootstrap {
            callback(Arc::clone(&lifecycle.provider)).await?;
        }
    }
}
```

This is for things that need ALL modules to be ready before they start — final validation, health check registration, scheduled task activation.

### ⑥ `adapter.build()` — build the Axum router

Only after all lifecycle hooks have succeeded does the framework call `adapter.build(Arc::new(http))`. This is where the Axum adapter converts compiled routes into Axum's `Router` with tower layers (compression, body limit, timeout). If the adapter fails, the already-initialized providers get `destroy_modules()` called before the error returns.

---

## Phase 2 — Serve (Axum is running)

`FrameworkApplication::listen_with_shutdown()` at `application.rs:310`:

```rust
pub async fn listen_with_shutdown(self, address, shutdown) -> Result {
    let FrameworkApplication { platform, initialized, .. } = self;

    let serving = platform.listen(address, Shutdown::new(shutdown)).await;
    let signal = serving.ok().copied().unwrap_or(ShutdownSignal::Custom("error"));

    let cleanup = shutdown_application(&initialized, signal).await;
    serving?;  // propagate platform error
    cleanup
}
```

Notice the order: `platform.listen()` runs FIRST, then `shutdown_application()` runs AFTER. The lifecycle hooks are never inside Axum's request handling — they bracket it entirely.

Inside the Axum adapter (`platform-axum/src/lib.rs:194`):

```rust
fn listen(self, address, shutdown) -> PlatformFuture<Result<ShutdownSignal, Self::Error>> {
    Box::pin(async move {
        let listener = tokio::net::TcpListener::bind(address).await?;

        let (sender, receiver) = tokio::sync::oneshot::channel();

        let graceful = async move {
            let signal = shutdown.wait().await;   // ← Ctrl-C or custom
            let _ = sender.send(signal);          // ← sends signal back
        };

        axum::serve(listener, self.router)
            .with_graceful_shutdown(graceful)
            .await?;

        Ok(receiver.await.unwrap_or(ShutdownSignal::Custom("platform")))
    })
}
```

The `oneshot::channel` is the only connection between the shutdown signal and the caller. The `graceful` future waits for the shutdown signal, then fires the oneshot sender. `axum::serve` stops accepting new connections, drains in-flight requests, and returns. Then `receiver.await` returns the shutdown signal back to Ironic's `listen_with_shutdown()`.

**The only lifecycle interaction inside Axum's handler stack** is the `RequestScope` injection, which happens per-request in each Axum handler closure (`route.rs:658`):

```rust
pub async fn execute(&self, route, context) -> Result<FrameworkResponse, HttpError> {
    if context.extension::<RequestScope>().is_none() {
        context.insert_extension(self.container.request_scope());
    }
    super::pipeline::execute(self, route, context).await
}
```

This creates a fresh per-request scope and inserts it into the `RequestContext` extensions. The middleware/guard/interceptor/handler chain resolves request-scoped providers from this scope, not from the container directly. But this is NOT a lifecycle hook — it's a DI concern. The lifecycle hooks run completely outside this path.

---

## Phase 3 — Shutdown (Axum has stopped)

Back in `listen_with_shutdown()`, after `platform.listen()` returns:

```rust
let cleanup = shutdown_application(&initialized, signal).await;
```

`shutdown_application()` at `application.rs:416`:

```rust
async fn shutdown_application(initialized, signal) -> Result {
    let mut first_error = None;

    // Phase 3a: OnApplicationShutdown in REVERSE order
    for lifecycle in initialized.iter().rev() {
        if let Some(callback) = &lifecycle.definition.application_shutdown {
            if let Err(error) = callback(Arc::clone(&lifecycle.provider), signal).await {
                first_error.get_or_insert_with(|| lifecycle_error(..., &error));
            }
        }
    }

    // Phase 3b: OnModuleDestroy in REVERSE order
    if let Err(error) = destroy_modules(initialized).await {
        first_error.get_or_insert(error);
    }

    first_error.map_or(Ok(()), Err)
}
```

Two sub-phases, both in reverse init order:

1. **OnApplicationShutdown** — runs first. Each handler receives the `ShutdownSignal` (Interrupt, Terminate, or Custom). This is for app-level cleanup: flush metrics, close background tasks, notify load balancers.

2. **OnModuleDestroy** — runs second. Each module cleans up its own resources: close database pools, flush caches, release locks.

The error strategy is deliberate: `first_error.get_or_insert_with(...)` means ALL callbacks are attempted, even if some fail. The first error is returned, but no callback is skipped.

---

## The complete timeline as a diagram

```
build()
  ├─ compile_module_graph()          → computes init + shutdown order
  ├─ build_http_application()        → builds Container, compiles routes
  ├─ initialize_eager_providers()    → resolves @injectable(eager)
  ├─ initialize_lifecycle()          → OnModuleInit called (forward order)
  │    └─ stores InitializedLifecycle vec
  ├─ bootstrap_application()         → OnApplicationBootstrap (forward order)
  └─ adapter.build(Arc::new(http))   → builds Axum router with tower layers
      │
      ▼
listen()
  └─ platform.listen(addr, shutdown)
       └─ axum::serve(listener, router)
            .with_graceful_shutdown(graceful)
            │
            ├── [request] → execute_route() → RequestScope injected → pipeline
            ├── [Ctrl-C]  → graceful future resolves → oneshot fires
            ├── [drain]   → in-flight requests complete
            └── [return]  → axum::serve returns
                │
                ▼
shutdown_application(initialized, signal)
  ├─ OnApplicationShutdown (REVERSE order)
  └─ destroy_modules() → OnModuleDestroy (REVERSE order)
      │
      ▼
  cleanup result returned
```

---

## Why this design

1. **Deterministic order** — the topological sort guarantees `DatabaseModule` starts before `UserModule`, and `UserModule` shuts down before `DatabaseModule`. No race conditions, no dependency guessing.

2. **Partial failure safety** — if `UserModule` fails init, `DatabaseModule` (already initialized) gets `OnModuleDestroy` called before the error propagates. The `initialized` vec tracks exactly what succeeded, so rollback is precise.

3. **Separate from Axum** — lifecycle hooks never touch the Axum middleware stack. They don't need to. Init runs before the listener binds. Shutdown runs after the listener returns. The `RequestScope` injection is the only framework concern inside the Axum handler, and that's DI, not lifecycle.

4. **Type-erased but type-safe** — the callbacks are `Arc<dyn Fn(ProviderValue) -> LifecycleFuture>`, which erases the concrete provider type. But the `LifecycleDefinitionBuilder` has a `PhantomData<T>` that forces the factory and callbacks to agree on `T`. The `downcast::<T>(value)` at call time is just a double-check — the types were already enforced by the builder.

5. **No global state** — the `initialized` vec is owned by `FrameworkApplication` and consumed by `shutdown_application`. Nothing is leaked. No static variables. No `Mutex` on the shutdown path.
