---
title: "Module graph compilation — how Ironic validates your entire app before it starts"
description: "A deep dive into graph validation, topological ordering, visibility scoping, and how Ironic catches circular dependencies, duplicates, and missing providers at compile time — not at 3 AM."
date: "2026-07-15"
author: "Ironic Team"
---

# Module graph compilation — how Ironic validates your entire app before it starts

Startup crashes are the worst kind of crash. They happen in production, at 1 AM, because a DI container couldn't find a dependency that was always going to be missing. Ironic takes a different approach: before any provider is constructed, before any HTTP route is registered, the framework compiles your entire module graph and proves it's correct.

The function at the center of this is `compile_module_graph` in `crates/ironic-core/src/lib.rs:543`. It accepts a single root `ModuleDefinition`, descends through every import, and either returns a `CompiledApplicationGraph` — a fully validated blueprint of your application — or a precise `ModuleError` telling you exactly what's wrong.

---

## What's inside a ModuleDefinition

Every module in Ironic implements the `Module` trait (`lib.rs:35`), which requires a single associated function:

```rust
fn definition() -> ModuleDefinition;
```

A `ModuleDefinition` (`lib.rs:113`) carries six pieces of static metadata:

- **`id: ModuleId`** — a type-based identity derived from `TypeId` and `type_name`, used for deduplication and diagnostics.
- **`imports: Vec<ModuleDefinitionFactory>`** — other modules this module depends on. Factories are lazily expanded during graph traversal.
- **`providers: Vec<ProviderDefinition>`** — DI provider registrations owned by this module (services, repositories, configuration values).
- **`controllers: Vec<ControllerDefinition>`** — HTTP controller definitions (routes + their provider backing).
- **`exports: Vec<ProviderKey>`** — the set of owned providers that importing modules are allowed to see.
- **`lifecycle: Vec<LifecycleDefinition>`** — `OnModuleInit` / `OnModuleDestroy` hooks tied to owned providers.
- **`global: bool`** — when `true`, the module's exports are visible to every module in the graph without explicit imports.

The builder API (`lib.rs:158`) provides a fluent interface: `.import::<OtherModule>()`, `.provider(...)`, `.export::<T>()`, `.controller(...)`, `.global()`, and `.build()`. There's also conditional composition with `.import_if(enabled)` and `.provider_if(enabled, ...)`.

---

## Topological sort: imports determine init order

`compile_module_graph` calls the recursive `discover` function (`lib.rs:647`) which performs a depth-first traversal with three-color marking:

- **`None`** — the module hasn't been visited yet. Visit its imports recursively, then mark it.
- **`Visiting`** — the module is currently on the DFS path. Seeing it again means **a cycle**.
- **`Visited`** — the module and all its descendants have been processed.

Each visited module is pushed onto an `order` vector in **post-order** (after imports are processed). This means `initialization_order` always lists the deepest dependency first and the root last — you initialize `DatabaseModule` before `UsersModule`, and `UsersModule` before `AppModule`. The `shutdown_order` is simply the reverse (`lib.rs:559`), ensuring that the root shuts down first and foundational modules are torn down last.

```
        ┌──────────────────────────────────────────┐
        │              AppModule (root)             │
        │  ┌─────────────────────────────────────┐  │
        │  │        imports: [UsersModule]        │  │
        │  └─────────────────────────────────────┘  │
        └──────────────────┬───────────────────────┘
                           │
              ┌────────────▼──────────────┐
              │        UsersModule        │
              │  ┌──────────────────────┐ │
              │  │ providers: [Repo,    │ │
              │  │             Service] │ │
              │  │ exports:   [Service] │ │
              │  │ imports:   [Database │ │
              │  │            Module]   │ │
              │  └──────────────────────┘ │
              └──────────┬───────────────┘
                         │
         ┌───────────────▼──────────────┐
         │        DatabaseModule        │
         │  ┌─────────────────────────┐ │
         │  │ providers: [Connection]  │ │
         │  │ exports:   [Connection]  │ │
         │  │ imports:   []            │ │
         │  └─────────────────────────┘ │
         └──────────────────────────────┘

  initialization_order: [DatabaseModule, UsersModule, AppModule]
  shutdown_order:       [AppModule, UsersModule, DatabaseModule]
```

---

## Provider visibility: who can see what

The compiler computes visibility inside `compile_module` (`lib.rs:765`). For each module, three sets of providers are visible:

1. **Own providers** — always visible (they belong to this module).
2. **Direct import exports** — providers explicitly `export`ed by modules listed in `imports`. If two imports export the same key, that's `AmbiguousImport`.
3. **Global module exports** — providers exported by modules marked `global: true`. These are visible to every module in the graph without any explicit import.

A module's own providers take priority over imported ones, so a local registration always shadows an import. The resulting `visible_providers` map (key → owning `ModuleId`) is stored on each `CompiledModule`.

---

## Circular import detection at compile time

The `discover` function rejects cycles during graph traversal. If it encounters a module that is already in the `Visiting` state on the current DFS path, it captures the cycle path and returns `ModuleError::ImportCycle` with the exact sequence (`lib.rs:656`). This means you get a compile-time error like:

```
RF_MODULE_IMPORT_CYCLE: importing `my_app::AuthModule` creates a cycle
path: [AuthModule, UsersModule, AuthModule]
```

The error includes the full cycle, so you know exactly which imports form the loop. This is fundamentally different from runtime DI cycle detection — the graph itself is proven acyclic before any code runs.

---

## Catching duplicates

Before any module is marked visited, `validate_local_duplicates` (`lib.rs:696`) scans the definition for three categories of collisions:

- **Duplicate imports** — the same module listed twice in `imports` → `ModuleError::DuplicateImport`.
- **Duplicate providers** — two providers with the same `ProviderKey` within one module → `ModuleError::DuplicateProvider`.
- **Duplicate controllers** — two controllers with the same type key within one module → `ModuleError::DuplicateController`.

There is also a cross-module duplicate check in `validate_controller_ownership` (`lib.rs:746`): two modules cannot own controllers of the same type. This guarantees unambiguous route-to-controller dispatch.

---

## Dependency validation: no missing or private references

Once visibility is computed for every module, `compile_module` calls `validate_dependencies` (`lib.rs:843`) on every provider and controller. For each declared dependency:

- If the dependency is **optional**, it's skipped — you're explicitly saying "I can live without this."
- If the key exists in `visible_providers`, it's satisfied.
- If an import **owns** the provider but hasn't exported it, the error is `ModuleError::PrivateProvider` — you can see that someone has it, but they won't share.
- If nobody in the visible graph owns it at all, the error is `ModuleError::MissingProvider`.

The distinction matters. `PrivateProvider` tells you "add `.export::<T>()` to the module that owns it." `MissingProvider` tells you "nobody registered it — import the right module or create the provider."

---

## A concrete example

Consider three modules:

```rust
struct DatabaseModule;
struct UserModule;
struct AuthModule;

struct PgConnection;
struct UserRepo;
struct AuthService;

impl Module for DatabaseModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .provider(ProviderDefinition::value(PgConnection))
            .export::<PgConnection>()
            .build()
    }
}

impl Module for UserModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .import::<DatabaseModule>()
            .provider(ProviderDefinition::factory::<UserRepo, _, _>(
                Scope::Singleton,
                vec![Dependency::required::<PgConnection>()],
                |resolver| async move {
                    Ok(UserRepo {
                        db: resolver.resolve().await?,
                    })
                },
            ))
            .export::<UserRepo>()
            .build()
    }
}

impl Module for AuthModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .import::<DatabaseModule>()
            .import::<UserModule>()
            .provider(ProviderDefinition::factory::<AuthService, _, _>(
                Scope::Singleton,
                vec![
                    Dependency::required::<PgConnection>(),
                    Dependency::required::<UserRepo>(),
                ],
                |resolver| async move {
                    Ok(AuthService {
                        db: resolver.resolve().await?,
                        users: resolver.resolve().await?,
                    })
                },
            ))
            .build()
    }
}
```

When you call `compile_module_graph(AuthModule::definition())`, the compiler:

1. **Discovers** the graph: `AuthModule → DatabaseModule + UserModule → DatabaseModule`. `DatabaseModule` is visited once, deduplicated naturally by the `Visited` check.
2. **Topologically sorts**: `initialization_order = [DatabaseModule, UserModule, AuthModule]`.
3. **Computes visibility for `AuthModule`**: own providers (`AuthService`) + `DatabaseModule` exports (`PgConnection`) + `UserModule` exports (`UserRepo`). All three are visible.
4. **Validates `AuthService`'s dependencies**: `PgConnection` exists (via `DatabaseModule` export). `UserRepo` exists (via `UserModule` export). No errors.
5. **Returns `CompiledApplicationGraph`** — proof the graph is sound.

If `UserModule` forgot to `.export::<UserRepo>()`, the compiler would return `ModuleError::PrivateProvider` pointing at `AuthModule → AuthService → UserRepo`, with the owner set to `UserModule`.

If `AuthModule` declared a dependency on `EmailService` that nobody registered, the compiler would return `ModuleError::MissingProvider`.

---

## Why compile-time validation matters

Every error in the `ModuleError` enum (`lib.rs:356`) carries structured data: the module identity, the provider or controller key, the full cycle path, and the owner modules. These aren't opaque stack traces — they're actionable directives. The compiler tells you *which module* has the problem, *which provider* is involved, and *what to do* about it.

By the time `build_http_application` (`lib.rs:573`) runs through the validated graph registering providers into a `ContainerBuilder`, the hard questions are already answered. There are no missing registrations to surface at 1 AM. The graph is acyclic. Every dependency is visible. Controller ownership is unambiguous. The `initialization_order` and `shutdown_order` are precomputed and deterministic.

You trade a few milliseconds of compile time for the certainty that your application — every import, every export, every lifecycle hook — forms a coherent whole before the first `tokio::spawn`.
