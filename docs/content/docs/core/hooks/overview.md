---
title: Lifecycle Hooks
description: The complete lifecycle hook system — 15 hooks covering startup, request, runtime, shutdown, and dynamic module phases.
---

# Lifecycle Hooks

Ironic provides **15 lifecycle hooks** that let you plug into every phase of an application's lifecycle.

## The full lifecycle

```
              ┌────────────────────┐
              │ OnModuleConfigure  │ ← Module graph compiled
              └────────┬───────────┘
                       ▼
              ┌────────────────────┐
              │ Eager Providers     │ ← Resolved bottom-up
              └────────┬───────────┘
                       ▼
              ┌────────────────────┐
              │ OnModuleInit       │ ← Per-module initialization
              └────────┬───────────┘
                       ▼
              ┌────────────────────┐
              │ OnApplicationBootstrap │ ← All modules ready
              └────────┬───────────┘
                       ▼
              ┌────────────────────┐
              │ OnServerReady      │ ← HTTP server bound
              └────────┬───────────┘
                       ▼
    ╔══════════════════════════════════════╗
    ║          SERVER RUNNING             ║
    ║                                     ║
    ║  ┌─ Request Arrives                 ║
    ║  │  ┌─────────────────────────────┐ ║
    ║  ├──│ OnRequestInit               │ ║ ← Request-scoped provider created
    ║  │  └─────────────┬───────────────┘ ║
    ║  │                ▼                 ║
    ║  │  ┌─────────────────────────────┐ ║
    ║  │  │ Middleware → Guards → ...   │ ║
    ║  │  │   ├─ OnGuardDenied          │ ║ ← Guard returns Deny
    ║  │  │   ├─ OnError                │ ║ ← Any unhandled error
    ║  │  │   └─ Handler                │ ║
    ║  │  └─────────────┬───────────────┘ ║
    ║  │                ▼                 ║
    ║  └──│ OnRequestDestroy             ║ ← Request scope ends
    ║     └─────────────────────────────┘ ║
    ╚══════════════════════════════════════╝
                       ▼
              ┌────────────────────┐
              │ BeforeShutdown     │ ← Signal received, server still accepting
              └────────┬───────────┘
                       ▼
              ┌────────────────────┐
              │ OnApplicationShutdown │ ← Reverse order cleanup
              └────────┬───────────┘
                       ▼
              ┌────────────────────┐
              │ OnModuleDestroy    │ ← Per-module cleanup (reverse)
              └────────┬───────────┘
                       ▼
              ┌────────────────────┐
              │ AfterShutdown      │ ← Final cleanup
              └────────────────────┘

    ╔══════════════════════════════════════╗
    ║       RUNTIME MODULE LIFECYCLE      ║
    ║                                     ║
    ║  ┌─ Module dynamically loaded       ║
    ║  │  ┌─────────────────────────────┐ ║
    ║  ├──│ OnModuleLoad               │ ║ ← Provider init for new module
    ║  │  └─────────────────────────────┘ ║
    ║  │                ...                ║
    ║  │  ┌─────────────────────────────┐ ║
    ║  ├──│ OnModuleUnload             │ ║ ← Provider cleanup before removal
    ║  │  └─────────────────────────────┘ ║
    ╚══════════════════════════════════════╝
```

## Startup hooks

### OnModuleConfigure

Runs during module graph compilation, **before any providers are built**. Receives the module's diagnostic name.

| When | Before everything |
|---|---|
| Use for | Dynamic route registration, conditional provider setup, validating module configuration |
| Trait | `fn on_module_configure(&self, module_name: &str) -> LifecycleFuture<'_>;` |

### OnModuleInit

Runs after a provider's module and dependencies are available.

| When | After DI resolves, before server starts |
|---|---|
| Use for | Database migrations, cache warmup, connection pool init |
| Trait | `fn on_module_init(&self) -> LifecycleFuture<'_>;` |

### OnApplicationBootstrap

Runs after **every** module's `OnModuleInit` succeeds. All dependencies are ready.

| When | All modules initialized, before server listens |
|---|---|
| Use for | Background tasks, cron jobs, cross-module health checks |
| Trait | `fn on_application_bootstrap(&self) -> LifecycleFuture<'_>;` |

### OnServerReady

Runs after the HTTP server binds to a port and is ready for connections.

| When | Server is listening |
|---|---|
| Use for | Self-health check, notify orchestrator, log bound address |
| Trait | `fn on_server_ready(&self) -> LifecycleFuture<'_>;` |

## Request hooks

### OnRequestInit

Runs when a request-scoped provider is first resolved within a request.

| When | New request begins |
|---|---|
| Use for | Per-request auth context setup, temp resource allocation, logging request ID |
| Trait | `fn on_request_init(&self, request_id: &str) -> LifecycleFuture<'_>;` |

### OnRequestDestroy

Runs when the request scope ends and the provider is about to be dropped.

| When | Request scope ends |
|---|---|
| Use for | Close temp connections, flush per-request metrics, release resources |
| Trait | `fn on_request_destroy(&self) -> LifecycleFuture<'_>;` |

## Runtime / error hooks

### OnError

Called on **every unhandled error** before exception filters run.

| When | Any pipeline error |
|---|---|
| Use for | Centralized error logging, Sentry/DataDog reporting, alerting on error codes |
| Trait | `fn on_error(&self, error_code: &str, error_message: &str) -> LifecycleFuture<'_>;` |

### OnGuardDenied

Called when any `Guard` returns `GuardDecision::Deny`.

| When | Auth check fails |
|---|---|
| Use for | Centralized auth failure logging, brute-force detection, rate-limit counters |
| Trait | `fn on_guard_denied(&self, guard_name: &str) -> LifecycleFuture<'_>;` |

## Shutdown hooks

### BeforeShutdown

Runs immediately after shutdown signal is received, **before** the server stops accepting connections.

| When | Shutdown starts |
|---|---|
| Use for | Drain connections, reject new requests, signal load balancers |
| Trait | `fn before_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;` |

### OnApplicationShutdown

Runs after serving stops, before module destruction. Receives the `ShutdownSignal`.

| When | Server stopped |
|---|---|
| Use for | Final cleanup, metrics snapshot |
| Trait | `fn on_application_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_>;` |

### OnModuleDestroy

Runs during cleanup in **reverse** successful-initialization order.

| When | Per-module destruction |
|---|---|
| Use for | Close database connections, flush buffers |
| Trait | `fn on_module_destroy(&self) -> LifecycleFuture<'_>;` |

### AfterShutdown

Runs after **all** `OnModuleDestroy` callbacks have completed.

| When | Everything cleaned up |
|---|---|
| Use for | Final metrics flush, last-chance cleanup, log shutdown duration |
| Trait | `fn after_shutdown(&self) -> LifecycleFuture<'_>;` |

## Registration

Lifecycle hooks are **automatically discovered** when a provider implements the trait and is registered via `providers` in `#[module(...)]`:

```rust
#[derive(Module)]
#[module(
    providers = [StatsReporter],
)]
pub struct TasksModule;

#[derive(Injectable)]
pub struct StatsReporter {
    service: Arc<BlogService>,
}

impl OnApplicationBootstrap for StatsReporter {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            // start background tasks
            Ok(())
        })
    }
}
```

For manual registration, use `LifecycleDefinition::builder()`:

```rust
LifecycleDefinition::builder::<StatsReporter>()
    .on_error()
    .guard_denied()
    .build()
```

## Execution order

| Phase | Direction | Order |
|---|---|---|
| `OnModuleConfigure` | Forward | Topological order (leaves first) |
| `OnModuleInit` | Forward | Topological order |
| `OnApplicationBootstrap` | Forward | Registration order |
| `OnServerReady` | Forward | Registration order |
| `OnRequestInit` | Per-request | Provider resolution order |
| `OnError` | Per-error | Registration order |
| `OnGuardDenied` | Per-deny | Registration order |
| `BeforeShutdown` | Forward | Registration order |
| `OnApplicationShutdown` | **Reverse** | Reverse registration order |
| `OnModuleDestroy` | **Reverse** | Reverse init order |
| `AfterShutdown` | **Reverse** | Reverse registration order |
| `OnRequestDestroy` | Per-request | Reverse init order |

## Error handling

**Startup failures:** If any startup hook fails (`OnModuleInit`, `OnApplicationBootstrap`, `OnServerReady`), the application aborts. All successfully initialized modules have their `OnModuleDestroy` called in reverse order — no resources are leaked.

**Shutdown failures:** Shutdown hooks run **best-effort** — all callbacks execute even if some fail. Only the first error is returned.

**Request failures:** Failed `OnRequestInit` prevents the handler from running. `OnError` fires on every unhandled error. Failed `OnGuardDenied` hooks are silently logged.

## ShutdownSignal

```rust
pub enum ShutdownSignal {
    Interrupt,            // Ctrl-C
    Terminate,            // SIGTERM
    Custom(&'static str), // Programmatic shutdown
}
```

## Choosing the right hook

| You want to... | Use |
|---|---|
| Validate module config before building | `OnModuleConfigure` |
| Initialize per-module resources | `OnModuleInit` |
| Start background tasks after everything is ready | `OnApplicationBootstrap` |
| Run a health check after the server binds | `OnServerReady` |
| Set up auth context per request | `OnRequestInit` |
| Report all errors to Sentry | `OnError` |
| Log every failed auth attempt | `OnGuardDenied` |
| Drain connections during shutdown | `BeforeShutdown` |
| Flush metrics after everything stops | `AfterShutdown` |
| Close database pools | `OnModuleDestroy` |
