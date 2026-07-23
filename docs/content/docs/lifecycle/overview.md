---
title: Overview
description: The Ironic lifecycle system — hooks at every phase of your application.
---

# Lifecycle Hooks Overview

Ironic provides **15 lifecycle hooks** that let you run code at specific moments during your application's startup, request handling, and shutdown.

## Why lifecycle hooks?

Lifecycle hooks let you separate cross-cutting concerns from your business logic:

- **Startup**: Connect to databases, run migrations, seed data
- **Request**: Initialize per-request context, clean up after response
- **Error handling**: Centralized error logging, metrics, alerting
- **Shutdown**: Close connections gracefully, flush metrics

## Execution order

```
COMPILE TIME
  │
  ▼
OnModuleConfigure   — Dynamic route registration, conditional setup
  │
  ▼
STARTUP
  │
  ▼
AsyncModuleInit     — Container-aware init (DB connections, migrations)
  │
  ▼
OnModuleInit        — Per-provider initialization (deps resolved)
  │
  ▼
OnApplicationBootstrap  — After all modules init, before server listens
  │
  ▼
OnServerReady       — Server is bound and accepting connections
  │
  ▼
RUNNING
  │
  ├──► OnRequestInit      — Per-request (first resolve in scope)
  ├──► OnError             — On unhandled error
  ├──► OnGuardDenied       — When a Guard denies access
  └──► OnRequestDestroy    — Per-request cleanup
  │
  ▼
SHUTDOWN
  │
  ▼
BeforeShutdown      — Before server stops accepting connections
  │
  ▼
OnModuleDestroy     — Per-module cleanup (reverse init order)
  │
  ▼
OnApplicationShutdown  — After all modules destroyed
  │
  ▼
AfterShutdown       — Final cleanup, metrics flush
```

## Registration

Each hook is registered via `LifecycleDefinitionBuilder`:

```rust
ModuleDefinition::builder::<DatabaseService>()
    .module_init()         // ← OnModuleInit
    .module_destroy()      // ← OnModuleDestroy
    .build()
```

## Execution guarantees

- **Startup hooks** run in dependency order (dependencies first)
- **Shutdown hooks** run in reverse initialization order
- **If a startup hook fails**, all previously succeeded hooks run their destroy in reverse
- **Request-scoped hooks** fire per-request scope
- **Errors in shutdown hooks** are logged but don't prevent other hooks from running

## Next steps

Read the hook page relevant to your use case:

- [OnModuleConfigure](/docs/lifecycle/on-module-configure) — Dynamic module setup before providers are built
- [OnModuleInit](/docs/lifecycle/on-module-init) — Initialize after dependencies are resolved
- [OnApplicationBootstrap](/docs/lifecycle/on-application-bootstrap) — After all modules init
- [OnServerReady](/docs/lifecycle/on-server-ready) — Server is accepting connections
- [OnRequestInit / OnRequestDestroy](/docs/lifecycle/on-request-init) — Per-request scope
- [OnError](/docs/lifecycle/on-error) — Centralized error handling
- [OnGuardDenied](/docs/lifecycle/on-guard-denied) — Auth failure tracking
- [BeforeShutdown / AfterShutdown](/docs/lifecycle/before-shutdown) — Graceful shutdown
- [OnModuleDestroy](/docs/lifecycle/on-module-destroy) — Release resources
- [OnApplicationShutdown](/docs/lifecycle/on-application-shutdown) — Last cleanup
- [OnModuleLoad / OnModuleUnload](/docs/lifecycle/on-module-load) — Dynamic modules
- [AsyncModuleInit](/docs/lifecycle/async-module-init) — Container-aware init
