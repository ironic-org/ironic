---
title: "Lifecycle orchestration — how Ironic boots, runs, and shuts down deterministically"
description: "A deep walk through Ironic's lifecycle trait hierarchy, topological init/shutdown ordering, eager providers, partial-failure rollback, and the full Ctrl-C → cleanup pipeline."
date: "2026-07-15"
author: "Ironic Team"
---

# Lifecycle orchestration — how Ironic boots, runs, and shuts down deterministically

Most frameworks have lifecycle hooks. Very few make the *order* of those hooks a first-class, compile-time guarantee. Ironic does. Its lifecycle pipeline flows through a strict sequence — module init, application bootstrap, application shutdown, module destroy — with an ordering derived from the same validated module graph that powers the DI container. Let's pull it apart from the traits down.

---

## The lifecycle trait hierarchy

Four traits define every lifecycle stage that a provider can opt into. They live in `crates/ironic-core/src/lifecycle.rs:32–53`:

```rust
pub trait OnModuleInit: Send + Sync + 'static {
    fn on_module_init(&self) -> LifecycleFuture<'_>;
}

pub trait OnApplicationBootstrap: Send + Sync + 'static {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_>;
}

pub trait OnModuleDestroy: Send + Sync + 'static {
    fn on_module_destroy(&self) -> LifecycleFuture<'_>;
}

pub trait OnApplicationShutdown: Send + Sync + 'static {
    fn on_application_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;
}
```

**OnModuleInit** fires once per provider, after the DI container has resolved it. This is where you open connection pools, pre-warm caches, or validate external configuration.

**OnApplicationBootstrap** fires *after every* `OnModuleInit` in the entire graph has completed — start to finish, zero failures. If you need to subscribe to an internal event bus that depends on other providers already being initialized, this is the hook.

**OnApplicationShutdown** is the mirror of bootstrap. It receives the `ShutdownSignal` that triggered shutdown — `Interrupt` for Ctrl-C, or `Custom` for caller-driven shutdown — so providers can distinguish between graceful and emergency teardown.

**OnModuleDestroy** is the mirror of `OnModuleInit`. Run in *reverse initialization order*, it drains connections, flushes buffers, and releases OS handles.

All hooks return `LifecycleFuture<'_>`, which is `Pin<Box<dyn Future<Output = Result<(), LifecycleError>> + Send + '_>>`. That `LifecycleError` wraps a safe diagnostic string — no raw error chains, because lifecycle errors are always user-facing in Ironic's structured error codes (see `ApplicationError::Lifecycle` at `application.rs:45–53`).

---

## How initialization order is computed

When you call `compile_module_graph(root)`, the framework performs a depth-first post-order walk over the module import tree (`lib.rs:647–694`). Each discovered module's imports are visited first; only after all transitive imports are visited does the module itself get pushed into the order vector. This is classic topological sort — imported-before-importer.

The resulting `CompiledApplicationGraph` stores this as `initialization_order: Vec<ModuleId>` (`lib.rs:317`). Shutdown order is simply the reverse:

```rust
let shutdown_order = order.iter().rev().copied().collect();
```

This appears at `lib.rs:559–560`. Every engineer who's had to debug "why does my database pool close before the query service" knows why this matters: the module graph encodes a dependency direction, and teardown must walk that graph backwards.

---

## Eager providers: the bootstrap prelude

Before lifecycle hooks ever run, the framework initializes *eager providers*. These are providers registered with `.eager()` in their `ProviderDefinition` — they get resolved during `initialize_eager_providers` (`application.rs:345–368`):

```rust
async fn initialize_eager_providers(
    graph: &CompiledApplicationGraph,
    container: &Container,
) -> Result<(), ApplicationError> {
    for module_id in graph.initialization_order() {
        let module = graph.module(*module_id).unwrap();
        for provider in module.providers().iter().filter(|provider| provider.is_eager()) {
            container.resolve_key(provider.key()).await.map_err(|error| {
                ApplicationError::EagerProvider { ... }
            })?;
        }
    }
    Ok(())
}
```

Providers are resolved in `initialization_order` — imports first, then the modules that depend on them. If eager resolution fails, the entire bootstrap is aborted before any lifecycle hook runs. This is important: eager providers are your "must have or don't start" resources, and Ironic treats them accordingly.

---

## Partial-failure rollback

Here's where the lifecycle pipeline gets genuinely subtle. Consider five modules: `[M1, M2, M3, M4, M5]` in initialization order. If `M3`'s `OnModuleInit` hook fails, what happens?

The answer is in `application.rs:206–213`:

```rust
let mut initialized = Vec::new();
if let Err(error) = initialize_lifecycle(&graph, &container, &mut initialized).await {
    let _ = destroy_modules(&initialized).await;
    return Err(error);
}
```

`initialize_lifecycle` pushes each successfully initialized provider into `initialized` *before* calling its `module_init` callback (`application.rs:387–396`). If a callback returns `Err`, the function aborts immediately, and the caller invokes `destroy_modules` on the `initialized` slice. Since `destroy_modules` iterates in `.rev()` (`application.rs:438`), the providers that *did* succeed get `OnModuleDestroy` in reverse order.

In our example: `M1` and `M2` have their `OnModuleInit` run and are pushed into `initialized`. `M3`'s init fails. The rollback calls `OnModuleDestroy` on `M2` then `M1`. `M4` and `M5` never touch the lifecycle at all — they were never resolved. This gives you clean, deterministic cleanup with no leaked connections, verified by the test `cleans_up_partially_initialized_applications` at `application.rs:715–739`.

The exact same rollback pattern applies to `bootstrap_application` and even platform build failures (`application.rs:210–212`, `217–219`).

---

## Shutdown signal propagation: Ctrl-C → cleanup

When you call `application.listen(addr)`, the method at `application.rs:291–303` constructs a shutdown future around `tokio::signal::ctrl_c()`. The sequence is:

1. **Listen opens**: `platform.listen(address, Shutdown::new(shutdown)).await` begins serving requests.
2. **Signal arrives**: Ctrl-C resolves the shutdown future. The platform's `listen` implementation returns `Ok(ShutdownSignal::Interrupt)`.
3. **Stop accepting**: The platform stops the listener and drains in-flight requests. From the test fake at `application.rs:648–659`, you can see this as the transition from `serve-stop` back to caller code.
4. **OnApplicationShutdown** fires first, iterating `initialized` in reverse. Each provider receives the signal. Shutdown errors are captured (`get_or_insert_with`) but don't prevent remaining providers from running their hooks (`application.rs:421–428`).
5. **OnModuleDestroy** fires next in the same reverse order, releasing resources (`application.rs:430–431`).
6. The first captured error — if any — is returned. Otherwise, `Ok(())`.

This "best-effort, first-error" strategy across both `shutdown_application` and `destroy_modules` at `application.rs:416–448` means a single misbehaving cleanup doesn't orphan every other resource.

---

## A concrete trace

Consider an `AppModule` that imports `UsersModule`, which imports `DatabaseModule` — exactly the graph in the test at `lib.rs:965–972`. Graph compilation produces:

```
initialization_order: [DatabaseModule, UsersModule, AppModule]
shutdown_order:      [AppModule, UsersModule, DatabaseModule]
```

Now add lifecycle hooks to `DatabaseModule` and `UsersModule`. Providers get eager-resolved bottom-up:

1. `DatabaseModule` providers are constructed
2. `UsersModule` providers are constructed

Then `initialize_lifecycle` walks the same order:

3. `DatabaseModule: OnModuleInit` fires
4. `UsersModule: OnModuleInit` fires

After all inits succeed, bootstrap fires:

5. `DatabaseModule: OnApplicationBootstrap`
6. `UsersModule: OnApplicationBootstrap`

The platform builds, the listener opens. Ctrl-C arrives. Shutdown fires reverse:

7. `UsersModule: OnApplicationShutdown(signal)`
8. `DatabaseModule: OnApplicationShutdown(signal)`
9. `UsersModule: OnModuleDestroy`
10. `DatabaseModule: OnModuleDestroy`

The test `runs_complete_lifecycle_in_deterministic_order` at `application.rs:662–713` validates every one of these steps — the events vector reads `[first-construct, second-construct, first-init, second-init, first-bootstrap, second-bootstrap, platform-build, listen, serve-stop, second-shutdown, first-shutdown, second-destroy, first-destroy]`.

---

## LifecycleDefinition — delayed registration for dynamic modules

All the hooks above are registered via `LifecycleDefinition`. A standard module registers lifecycle callbacks declaratively in its `ModuleDefinition::builder` through the `.lifecycle()` method (`lib.rs:228–232`). But for dynamic modules — those constructed at runtime with `import_definition` — `LifecycleDefinition` serves a different purpose.

The `LifecycleDefinitionBuilder<T>` (`lifecycle.rs:105–176`) decouples lifecycle registration from module declaration. You can construct a `LifecycleDefinition` anywhere, for any provider, at any time before building the application graph:

```rust
LifecycleDefinition::builder::<MyProvider>()
    .module_init()
    .application_bootstrap()
    .module_destroy()
    .application_shutdown()
    .build()
```

Each builder method — `module_init()`, `application_bootstrap()`, etc. — wraps the concrete provider behind an `Arc<dyn Fn(ProviderValue) -> LifecycleFuture>` with a downcast. The `downcast` helper at `lifecycle.rs:178–185` performs type-safe narrowing from `ProviderValue` to `Arc<T>`, returning a `LifecycleError` on mismatch. This is the only "runtime" step in the pipeline — the downcast — and it's a single `Any::downcast_ref` per callback.

Dynamic modules can compose their lifecycle definitions in-process, register callbacks at runtime, and still participate in the same deterministic `initialization_order` / `shutdown_order` sequence as statically declared modules.

---

## Why determinism matters

Ironic's lifecycle is not a best-effort event bus. It's a strictly ordered pipeline with three properties:

1. **Compile-time ordering**: `initialization_order` and `shutdown_order` are computed once during `compile_module_graph` and never recomputed. The order is always imports-before-importers.

2. **Atomic rollback**: Partial initialization failures cascade cleanly. If module N of M fails, modules 1 through N-1 are destroyed in reverse order. Nothing leaks.

3. **Single signal path**: `Ctrl-C → ShutdownSignal → stop accept → drain in-flight → OnApplicationShutdown → OnModuleDestroy`. No goroutines polling channels, no race between shutdown and cleanup — just one async pipeline from signal to final `drop`.

These guarantees mean you can reason about startup and shutdown the same way you'd reason about a function call stack. Resources are acquired in dependency order and released in the reverse. Every time.
