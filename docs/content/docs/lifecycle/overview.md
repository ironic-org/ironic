---
title: Lifecycle Hooks
description: Complete reference to all 15 lifecycle hooks — when they fire, what they're for, and how to use them.
---

# Lifecycle Hooks

Ironic provides **15 lifecycle hooks** that let you run code at specific
moments during startup, request handling, and shutdown.

## Visual Timeline

```
                        ┌─────────────────────────────────────────────┐
                        │         APPLICATION LIFECYCLE               │
                        └─────────────────────────────────────────────┘

  COMPILE   OnModuleConfigure ─── Dynamic config, conditional routes
     │
     ▼
  STARTUP   AsyncModuleInit ───── Container-aware async init
     │
     ▼        OnModuleInit ──────── Per-provider init (deps ready)
     │
     ▼        OnApplicationBootstrap ─ All modules initialized
     │
     ▼        OnServerReady ───────── Server is accepting traffic
     │
     ▼
  RUNNING    OnRequestInit ──────── New HTTP request starts
     │         │
     │         ├── OnError ───────── Unhandled error occurred
     │         ├── OnGuardDenied ─── Guard rejected request
     │         │
     │         ▼
     │        OnRequestDestroy ───── Request completes
     │
     ▼
  SHUTDOWN   BeforeShutdown ─────── Stop accepting connections
     │
     ▼        OnModuleDestroy ────── Per-module cleanup (reverse order)
     │
     ▼        OnApplicationShutdown ─ After all modules destroyed
     │
     ▼        AfterShutdown ──────── Final flush, metrics, exit
                        ┌─────────────────────────────────────────────┐
```

## Quick Registration

```rust
#[derive(Module)]
#[module(
    providers = [DatabaseService, MetricsService, AuditLogger],
    async_init = [DatabaseService],
    lifecycle_init = [DatabaseService, MetricsService],
    lifecycle_bootstrap = [AuditLogger],
    lifecycle_destroy = [DatabaseService],
    lifecycle_shutdown = [MetricsService, AuditLogger],
    lifecycle_server_ready = [HealthChecker],
    lifecycle_error = [ErrorReporter],
    lifecycle_guard_denied = [AuditLogger],
    lifecycle_request_init = [RequestTracer],
    lifecycle_request_destroy = [CleanupService],
    lifecycle_before_shutdown = [GracefulShutdown],
    lifecycle_after_shutdown = [MetricsFlusher],
    lifecycle_module_load = [DynamicLoader],
    lifecycle_module_unload = [DynamicLoader],
    lifecycle_configure = [RouteRegistrar],
)]
pub struct AppModule;
```

---

## Phase 1: Compile & Configure

### 1. `OnModuleConfigure`

**When:** During module graph compilation, before any providers are built.

**Purpose:** Dynamic route registration, conditional imports, environment-specific setup.

```rust
use ironic::prelude::*;

pub struct RouteRegistrar;

impl OnModuleConfigure for RouteRegistrar {
    fn configure(&self, module: &mut ModuleDefinitionBuilder) {
        if std::env::var("ENABLE_ADMIN").is_ok() {
            module.import::<AdminModule>();
        }
    }
}
```

**What you can do here:**
- Conditionally import modules based on environment variables
- Register routes that are only available in certain configurations
- Modify the module definition before it's compiled

---

## Phase 2: Startup

### 2. `AsyncModuleInit`

**When:** After the container is built but before singletons are resolved.
The container is available for lookups.

**Purpose:** Async initialization that needs the container (DB connections,
external service clients, config fetching).

```rust
use ironic::prelude::*;

#[derive(Injectable)]
pub struct DatabaseService;

impl AsyncModuleInit for DatabaseService {
    async fn async_init(&self, container: &Container) -> Result<(), LifecycleError> {
        let config = container
            .resolve::<DatabaseConfig>()
            .await
            .map_err(|e| LifecycleError::new(format!("config not found: {e}")))?;

        let pool = PgPool::connect(&config.url)
            .await
            .map_err(|e| LifecycleError::new(format!("db connect failed: {e}")))?;

        // Store the pool for later use
        container
            .resolve::<DatabasePoolStorage>()
            .await?
            .set_pool(pool);

        Ok(())
    }
}
```

**Key difference from `OnModuleInit`:** `AsyncModuleInit` receives the
`&Container` so it can resolve other providers. `OnModuleInit` does not.

### 3. `OnModuleInit`

**When:** After eager providers are constructed and forward refs are resolved.
All dependencies are ready.

**Purpose:** Run migrations, seed data, start background tasks — anything
that depends on fully constructed services.

```rust
use ironic::prelude::*;

#[derive(Injectable)]
pub struct DatabaseService {
    pool: Arc<DatabasePool>,
}

impl OnModuleInit for DatabaseService {
    async fn on_module_init(&self) -> Result<(), LifecycleError> {
        // Dependencies are fully resolved — safe to use
        self.pool.run_migrations().await?;

        tracing::info!("database migrations complete");
        Ok(())
    }
}
```

**Ordering:** Modules initialize in dependency order. If module A imports
module B, B's `OnModuleInit` runs before A's.

### 4. `OnApplicationBootstrap`

**When:** After ALL modules have completed their `OnModuleInit` hooks.
Before the server starts listening.

**Purpose:** Cross-module coordination, warm caches, register services
with external registries.

```rust
use ironic::prelude::*;

pub struct AuditLogger;

impl OnApplicationBootstrap for AuditLogger {
    async fn on_application_bootstrap(&self) -> Result<(), LifecycleError> {
        tracing::info!("all modules initialized — bootstrapping complete");
        Ok(())
    }
}
```

### 5. `OnServerReady`

**When:** The HTTP server is bound to its address and accepting connections.

**Purpose:** Health check registration, service discovery announcements,
metrics initialization.

```rust
use ironic::prelude::*;

pub struct HealthChecker;

impl OnServerReady for HealthChecker {
    async fn on_server_ready(&self) -> Result<(), LifecycleError> {
        tracing::info!("server is ready and accepting connections");
        // Register with service discovery
        register_with_consul().await?;
        Ok(())
    }
}
```

---

## Phase 3: Request Handling

### 6. `OnRequestInit`

**When:** Every HTTP request, before the controller handler runs.
The `RequestScope` has been created.

**Purpose:** Initialize per-request state, extract user context, setup tracing.

```rust
use ironic::prelude::*;

pub struct RequestTracer;

impl OnRequestInit for RequestTracer {
    async fn on_request_init(&self, context: &mut RequestContext) {
        let request_id = uuid::Uuid::new_v4().to_string();
        context.insert(RequestId(request_id));

        tracing::Span::current().record("request_id", &request_id);
    }
}
```

### 7. `OnRequestDestroy`

**When:** After the response is sent, during request cleanup.

**Purpose:** Release per-request resources, flush logs, cleanup.

```rust
use ironic::prelude::*;

pub struct CleanupService;

impl OnRequestDestroy for CleanupService {
    async fn on_request_destroy(&self) {
        tracing::debug!("request resources cleaned up");
    }
}
```

### 8. `OnError`

**When:** An unhandled error escapes from a controller or service.

**Purpose:** Centralized error logging, error reporting, metrics.

```rust
use ironic::prelude::*;

pub struct ErrorReporter;

impl OnError for ErrorReporter {
    async fn on_error(&self, error: &HttpError, context: &RequestContext) {
        tracing::error!(
            error = %error,
            path = %context.request().uri(),
            "unhandled error"
        );

        metrics::counter!("http_errors_total", 1);
    }
}
```

### 9. `OnGuardDenied`

**When:** A `Guard` returns `GuardDecision::Deny`.

**Purpose:** Audit logging, rate limit tracking, suspicious activity detection.

```rust
use ironic::prelude::*;

pub struct AuditLogger;

impl OnGuardDenied for AuditLogger {
    async fn on_guard_denied(&self, context: &RequestContext, guard: &str) {
        tracing::warn!(
            guard = %guard,
            path = %context.request().uri(),
            "access denied"
        );
    }
}
```

---

## Phase 4: Shutdown

### 10. `BeforeShutdown`

**When:** A shutdown signal has been received. The server stops accepting
new connections, but in-flight requests continue.

**Purpose:** Prepare for shutdown — drain connections, notify load balancers.

```rust
use ironic::prelude::*;

pub struct GracefulShutdown;

impl BeforeShutdown for GracefulShutdown {
    async fn before_shutdown(&self, signal: ShutdownSignal) {
        tracing::info!("received shutdown signal: {signal:?}");

        // Notify load balancer to stop sending traffic
        mark_unhealthy().await;

        // Allow in-flight requests to complete
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

### 11. `OnModuleDestroy`

**When:** During shutdown, after the server has drained. Runs in reverse
initialization order (last initialized = first destroyed).

**Purpose:** Close connections, release resources, flush data.

```rust
use ironic::prelude::*;

#[derive(Injectable)]
pub struct DatabaseService {
    pool: Arc<DatabasePool>,
}

impl OnModuleDestroy for DatabaseService {
    async fn on_module_destroy(&self) -> Result<(), LifecycleError> {
        self.pool.close().await;
        tracing::info!("database connections closed");
        Ok(())
    }
}
```

**Important:** Errors in `OnModuleDestroy` are logged but do NOT prevent
other destroy hooks from running. This ensures maximum cleanup.

### 12. `OnApplicationShutdown`

**When:** After all modules have been destroyed.

**Purpose:** Final cleanup that must happen after everything else.

```rust
pub struct MetricsService;

impl OnApplicationShutdown for MetricsService {
    async fn on_application_shutdown(&self) {
        metrics::flush();
        tracing::info!("metrics flushed — shutdown complete");
    }
}
```

### 13. `AfterShutdown`

**When:** After `OnApplicationShutdown`. This is the LAST hook to fire.

**Purpose:** Guaranteed final flush — log sinks, metrics exporters.

```rust
pub struct MetricsFlusher;

impl AfterShutdown for MetricsFlusher {
    async fn after_shutdown(&self) {
        // Guaranteed to run even if other shutdown hooks fail
        tracing::info!("goodbye");
    }
}
```

---

## Phase 5: Dynamic Module Loading

### 14. `OnModuleLoad`

**When:** A lazy module is loaded at runtime via `ModuleRef::load::<T>()`.

**Purpose:** Initialize resources when a dynamic module is activated.

```rust
pub struct DynamicLoader;

impl OnModuleLoad for DynamicLoader {
    async fn on_module_load(&self, module_id: &ModuleId) {
        tracing::info!("module loaded: {}", module_id.type_name());
    }
}
```

### 15. `OnModuleUnload`

**When:** A dynamically loaded module is being unloaded.

**Purpose:** Clean up resources when a dynamic module is deactivated.

```rust
pub struct DynamicLoader;

impl OnModuleUnload for DynamicLoader {
    async fn on_module_unload(&self, module_id: &ModuleId) {
        tracing::info!("module unloaded: {}", module_id.type_name());
    }
}
```

---

## Hook Reference Table

| # | Hook | Timing | Has Container? | Can Fail? | Best For |
|---|------|--------|---------------|-----------|----------|
| 1 | `OnModuleConfigure` | Compilation | ❌ | ❌ | Conditional routes |
| 2 | `AsyncModuleInit` | Startup (early) | ✅ | ✅ | DB connections |
| 3 | `OnModuleInit` | Startup | ❌ | ✅ | Migrations, seeding |
| 4 | `OnApplicationBootstrap` | Startup (last) | ❌ | ✅ | Cross-module setup |
| 5 | `OnServerReady` | Server listening | ❌ | ✅ | Service discovery |
| 6 | `OnRequestInit` | Request start | ✅ (context) | ❌ | Tracing, user context |
| 7 | `OnRequestDestroy` | Request end | ❌ | ❌ | Cleanup |
| 8 | `OnError` | Error occurs | ✅ (context) | ❌ | Error reporting |
| 9 | `OnGuardDenied` | Guard denies | ✅ (context) | ❌ | Audit logging |
| 10 | `BeforeShutdown` | Shutdown signal | ❌ | ✅ | Drain connections |
| 11 | `OnModuleDestroy` | Shutdown (middle) | ❌ | ✅ (logged) | Close resources |
| 12 | `OnApplicationShutdown` | Shutdown (late) | ❌ | ✅ (logged) | Final cleanup |
| 13 | `AfterShutdown` | Shutdown (last) | ❌ | ❌ | Metrics flush |
| 14 | `OnModuleLoad` | Dynamic load | ❌ | ✅ | Init lazy modules |
| 15 | `OnModuleUnload` | Dynamic unload | ❌ | ✅ | Cleanup lazy modules |

## Execution Guarantees

### Startup Order

```
Modules are initialized in TOPOLOGICAL ORDER:
  Module A (no deps) ──▶ OnModuleInit A
  Module B (depends on A) ──▶ OnModuleInit B
```

### Shutdown Order

```
Modules are destroyed in REVERSE order:
  Module B ──▶ OnModuleDestroy B
  Module A ──▶ OnModuleDestroy A
```

### Error Handling

```
Startup failure:
  Module B fails OnModuleInit
  └─▶ Module A's OnModuleDestroy runs (cleanup)
  └─▶ Error is returned from Application::build()

Shutdown failure:
  Module B's OnModuleDestroy fails
  └─▶ Error is LOGGED (not returned)
  └─▶ Module A's OnModuleDestroy STILL RUNS
  └─▶ All modules get cleanup chance
```

## Registration via `#[derive(Module)]`

```rust
#[derive(Module)]
#[module(
    // Startup
    async_init = [DatabaseService],        // AsyncModuleInit
    lifecycle_init = [DatabaseService],    // OnModuleInit
    lifecycle_bootstrap = [AuditLogger],   // OnApplicationBootstrap
    lifecycle_server_ready = [HealthCheck],// OnServerReady

    // Request
    lifecycle_request_init = [Tracer],     // OnRequestInit
    lifecycle_request_destroy = [Cleaner], // OnRequestDestroy
    lifecycle_error = [ErrorReporter],     // OnError
    lifecycle_guard_denied = [AuditLogger],// OnGuardDenied

    // Shutdown
    lifecycle_before_shutdown = [Drainer], // BeforeShutdown
    lifecycle_destroy = [DatabaseService], // OnModuleDestroy
    lifecycle_shutdown = [MetricsFlusher], // OnApplicationShutdown
    lifecycle_after_shutdown = [Finalizer],// AfterShutdown

    // Dynamic
    lifecycle_module_load = [Loader],      // OnModuleLoad
    lifecycle_module_unload = [Loader],    // OnModuleUnload

    // Config
    lifecycle_configure = [Registrar],     // OnModuleConfigure
)]
pub struct AppModule;
```

## Registration via Builder

```rust
ModuleDefinition::builder::<MyModule>()
    // Register each lifecycle hook individually
    .lifecycle(
        LifecycleDefinition::builder::<DatabaseService>()
            .module_init()           // OnModuleInit
            .module_destroy()        // OnModuleDestroy
            .build(),
    )
    .lifecycle(
        LifecycleDefinition::builder::<ErrorReporter>()
            .on_error()              // OnError
            .build(),
    )
    .build();
```

## Testing Lifecycle Hooks

```rust
use ironic::testing::*;

#[tokio::test]
async fn test_on_module_init_runs() {
    let app = TestApplication::builder()
        .module(AppModule::definition())
        .build()
        .await;

    // OnModuleInit ran during build
    // Verify side effects (e.g., migrations applied)
    let db = app.container().resolve::<DatabaseService>().await?;
    assert!(db.is_initialized());
}
```

## Common Patterns

### Pattern 1: Self-Registering Health Checks

```rust
#[derive(Injectable)]
pub struct HealthCheck {
    registry: Arc<HealthRegistry>,
}

impl OnServerReady for HealthCheck {
    async fn on_server_ready(&self) -> Result<(), LifecycleError> {
        self.registry.register("database", || self.check_db()).await;
        self.registry.register("cache", || self.check_cache()).await;
        Ok(())
    }
}
```

### Pattern 2: Graceful Draining

```rust
#[derive(Injectable)]
pub struct ConnectionDrainer {
    pool: Arc<DatabasePool>,
}

impl BeforeShutdown for ConnectionDrainer {
    async fn before_shutdown(&self, signal: ShutdownSignal) {
        tracing::info!("draining connections (timeout: {:?})", signal.timeout());
        tokio::select! {
            _ = self.pool.drain() => {},
            _ = tokio::time::sleep(signal.timeout()) => {
                tracing::warn!("drain timed out");
            }
        }
    }
}
```

### Pattern 3: Request Tracing

```rust
#[derive(Injectable)]
pub struct RequestTracer;

impl OnRequestInit for RequestTracer {
    async fn on_request_init(&self, ctx: &mut RequestContext) {
        let span = tracing::info_span!(
            "request",
            method = %ctx.request().method(),
            path = %ctx.request().uri(),
        );
        ctx.insert(span);
    }
}
```

## What you learned

- [x] All 15 lifecycle hooks and when they fire
- [x] Startup: OnModuleConfigure → AsyncModuleInit → OnModuleInit → OnApplicationBootstrap → OnServerReady
- [x] Request: OnRequestInit → handler → OnError/OnGuardDenied → OnRequestDestroy
- [x] Shutdown: BeforeShutdown → OnModuleDestroy → OnApplicationShutdown → AfterShutdown
- [x] Dynamic: OnModuleLoad / OnModuleUnload for lazy modules
- [x] Execution guarantees: topological order, reverse shutdown, error isolation
- [x] Registration via `#[derive(Module)]` attributes or `LifecycleDefinition::builder()`
