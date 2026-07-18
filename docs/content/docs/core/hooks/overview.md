---
title: Lifecycle Hooks
description: The complete lifecycle hook system — 15 hooks covering startup, request, runtime, shutdown, and dynamic module phases.
---

# Lifecycle Hooks

## What is a lifecycle hook?

A lifecycle hook is a function that Ironic calls automatically at a specific moment in your application's life. You implement a trait, and the framework calls it at the right time — no manual invocation, no polling, no configuration files.

**Real-world analogy:** Think of a restaurant. The chef doesn't manually turn on the lights, unlock the door, or count the cash register at the end of the night. Each task happens at a specific phase — opening, serving customers, closing. Lifecycle hooks are your restaurant checklist, automated.

```
9:00 AM → OnModuleInit (unlock, turn on lights)
9:30 AM → OnApplicationBootstrap (final prep check)
9:45 AM → OnServerReady (flip "Open" sign)
──────────────────────────────────────────
   ALL DAY → Requests flow (customers served)
──────────────────────────────────────────
10:00 PM → BeforeShutdown (last call, no new orders)
10:05 PM → OnApplicationShutdown (kitchen closes)
10:10 PM → OnModuleDestroy (lights off)
10:15 PM → AfterShutdown (lock door, count register)
```

Without hooks, you'd have to manually call `seed_database()`, `start_cron()`, and `close_pools()` at the right time. With hooks, you just implement the traits and register them — Ironic handles the timing.

## How it works under the hood

Every lifecycle hook follows the same pattern under the hood:

```
1. YOU: Implement a trait on your provider
   impl OnModuleInit for MyService { ... }

2. YOU: Register it in #[module(...)]
   #[module(providers = [MyService], lifecycle_init = [MyService])]

3. Ironic's macro: Generates the wiring code
   .lifecycle( LifecycleDefinition::builder::<MyService>().module_init().build() )

4. At boot: Container resolves MyService, stores it as a ProviderValue

5. At the right moment: framework calls the type-erased callback
   callback(provider_value) → downcast to Arc<MyService> → calls on_module_init()
```

**Under the hood details:**

```rust
// What the macro generates (simplified):
LifecycleDefinition::builder::<MyService>()
    .module_init()   // only if MyService: OnModuleInit
    .build()
```

The `module_init()` method only compiles if `MyService` implements `OnModuleInit`. If you add a type to `lifecycle_init` that doesn't implement the trait, you get a **compile-time error** — not a runtime surprise.

The type-erased callback stores a closure:
```rust
Arc::new(|provider: ProviderValue| {
    Box::pin(async move {
        let svc = provider.downcast::<MyService>()?; // type-checked at registration
        svc.on_module_init().await                    // calls your code
    })
})
```

When the framework reaches `OnModuleInit` phase, it iterates over all registered lifecycle definitions, resolves each provider from the DI container, and invokes each callback in order. If any callback returns an error, the framework runs `OnModuleDestroy` in reverse for all successful providers — no half-initialized state.

## The Complete Visual Timeline

```
TIME    PHASE                           ORDER
────    ─────                           ─────
T+0s    OnModuleConfigure               Forward (leaves→root)
        │  Validates module config
        ▼
T+1s    OnModuleInit                    Forward (leaves→root)
        │  Each module sets up its resources
        ▼
T+3s    OnApplicationBootstrap          Forward
        │  Cross-module setup, cron jobs
        ▼
T+5s    OnServerReady                   Forward
        │  Server bound, health checks pass
        ▼
 ╔══════════════════════════════════════════════════╗
 ║              SERVER RUNNING                     ║
 ║                                                ║
 ║   Per Request:                                 ║
 ║     OnRequestInit → [Guards→Handler] → OnRequestDestroy
 ║                                                ║
 ║   On Error:                                    ║
 ║     OnError fires before exception filters     ║
 ║                                                ║
 ║   On Auth Fail:                                ║
 ║     OnGuardDenied fires per denial             ║
 ╚══════════════════════════════════════════════════╝
        │
T+N    BeforeShutdown                 Forward
        │  Signal received, server STILL accepting
        ▼
T+N+5s OnApplicationShutdown          Reverse
        │  Server stopped
        ▼
T+N+10s OnModuleDestroy               Reverse (root→leaves)
        │  Cleanup per module
        ▼
T+N+15s AfterShutdown                 Reverse
        │  Final flush, last-chance work
        ▼
       PROCESS EXITS
```

## Why you need lifecycle hooks

Without hooks, you'd write code like this:

```rust
#[ironic::main]
async fn main() {
    // Manual startup — easy to forget something
    seed_database().await;        // ← must run before server
    start_cron_jobs().await;      // ← must run after DB is ready
    warm_cache().await;           // ← depends on services

    let app = FrameworkApplication::builder()...build().await?;
    app.listen("0.0.0.0:3000").await?;

    // Manual shutdown — error-prone
    close_connections().await;    // ← what if it panics?
    flush_metrics().await;        // ← what order?
}
```

**Problems with manual approach:**
- Order dependency: what runs before what?
- Error handling: what if one fails? Do others still run?
- Shutdown: reverse order isn't intuitive
- Module ownership: which module owns which cleanup?

With hooks, Ironic handles all of this:

```rust
#[derive(Module)]
#[module(
    providers = [Database, Cache, CronJobs],
    lifecycle_init = [Database, Cache],
    lifecycle_bootstrap = [CronJobs],
)]
struct AppModule;

// That's it. Ironic handles the order, error handling, and shutdown.
```

## When to use each hook

### Startup Hooks

#### `OnModuleInit` — Initialize YOUR module's resources

**Why:** Every module needs to set itself up before it can serve requests.

**When:** After your module's dependencies are resolved by DI, before any business logic runs.

**How:**
```rust
impl OnModuleInit for Database {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let conn = connect_to_db(&self.url).await?;
            self.pool.set(conn).await;
            Ok(())
        })
    }
}
```

**Under the hood:** The framework topologically sorts modules by their imports. Leaf modules (no dependencies) initialize first. Root module initializes last. This way, when `UsersModule` initializes, `DatabaseModule` is already ready.

---

#### `OnApplicationBootstrap` — Cross-module startup work

**Why:** Some tasks need EVERY module to be ready. You can't start cron jobs if the database module hasn't finished `OnModuleInit`.

**When:** After ALL modules have completed `OnModuleInit`.

**How:**
```rust
impl OnApplicationBootstrap for StatsReporter {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            scheduling::cron("0 * * * * *", || {
                // All modules are ready — safe to query anything
                async move { report_stats().await }
            });
            Ok(())
        })
    }
}
```

**Under the hood:** The framework iterates over every successfully initialized provider and calls `on_application_bootstrap` in registration order. If any fail, all `OnModuleDestroy` hooks run in reverse for every initialized provider — clean rollback.

---

#### `OnServerReady` — Server is bound, but not serving yet

**Why:** You want to run a self-health check or notify your orchestrator that the server is about to start.

**When:** After the HTTP server binds to a port, RIGHT before the first request is accepted.

**How:**
```rust
impl OnServerReady for ReadinessProbe {
    fn on_server_ready(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            // Call your own /health endpoint
            let resp = reqwest::get("http://localhost:3000/health").await?;
            tracing::info!("Self-check: {}", resp.status());
            Ok(())
        })
    }
}
```

---

#### `OnModuleConfigure` — Validate BEFORE anything is built

**Why:** Catch configuration errors before expensive initialization (DB connections, pool creation).

**When:** During module graph compilation, BEFORE any providers are constructed.

**How:**
```rust
impl OnModuleConfigure for FeatureGate {
    fn on_module_configure(&self, module_name: &str) -> LifecycleFuture<'_> {
        Box::pin(async move {
            if std::env::var("ENABLE_EXPERIMENTAL").is_err() {
                tracing::info!("{}: experimental features disabled", module_name);
            }
            Ok(())
        })
    }
}
```

**Under the hood:** This runs inside `compile_module_graph()`. The module graph is assembled, validated, and `OnModuleConfigure` fires for each module with a lifecycle definition. If it fails, the app never starts — all before any provider is constructed.

---

### Request Hooks

#### `OnRequestInit` — One-time setup per HTTP request

**Why:** You want to initialize something ONCE per request (auth context, temp file, transaction), not on every handler invocation.

**When:** The first time a request-scoped provider is resolved within an HTTP request.

**How:**
```rust
impl OnRequestInit for RequestTracker {
    fn on_request_init(&self, request_id: &str) -> LifecycleFuture<'_> {
        Box::pin(async move {
            tracing::info!(request_id, "request started");
            self.start_time.set(Instant::now());
            Ok(())
        })
    }
}
```

#### `OnRequestDestroy` — One-time cleanup per HTTP request

**Why:** Clean up what `OnRequestInit` created. Guaranteed to run even if the handler panics.

**When:** When the request scope ends (after the response is sent).

**How:**
```rust
impl OnRequestDestroy for RequestTracker {
    fn on_request_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let elapsed = self.start_time.get().elapsed();
            tracing::info!("request completed in {:?}", elapsed);
            Ok(())
        })
    }
}
```

**Under the hood:** These are request-scoped providers. The DI container creates a `RequestScope` for each HTTP request. When a provider implementing `OnRequestInit` is first resolved, the framework calls it. When the scope is dropped, `OnRequestDestroy` runs.

---

### Runtime / Error Hooks

#### `OnError` — Global error reporter

**Why:** Instead of adding `tracing::error!()` to every handler, register ONE error reporter that catches everything.

**When:** On EVERY unhandled error, BEFORE exception filters transform the response.

**How:**
```rust
impl OnError for SentryReporter {
    fn on_error(&self, error_code: &str, error_message: &str) -> LifecycleFuture<'_> {
        Box::pin(async move {
            if error_code.starts_with("DB_") || error_code.starts_with("REDIS_") {
                self.sentry.capture_error(error_code, error_message).await;
            }
            Ok(())
        })
    }
}
```

**Under the hood:** When the pipeline encounters an `Err(...)` result, it calls all registered `OnError` hooks BEFORE any exception filter tries to catch it. This means you always see the raw error, even if a filter later transforms it into a clean JSON response.

---

#### `OnGuardDenied` — Auth failure central logger

**Why:** Instead of logging in every guard, log all auth failures from one place.

**When:** When ANY `Guard` returns `GuardDecision::Deny`.

**How:**
```rust
impl OnGuardDenied for BruteForceDetector {
    fn on_guard_denied(&self, guard_name: &str) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let count = self.counters.entry(guard_name).or_insert(0);
            *count += 1;
            if *count > 10 {
                tracing::error!(guard = guard_name, count, "brute force detected");
            }
            Ok(())
        })
    }
}
```

**Under the hood:** The pipeline iterates through guards sequentially. When one returns `Deny`, it calls all `OnGuardDenied` hooks with the guard's name, then short-circuits to return 403.

---

### Shutdown Hooks

#### `BeforeShutdown` — Graceful drain

**Why:** When Kubernetes sends SIGTERM, you have ~30 seconds to clean up. Don't waste that time — start draining immediately.

**When:** Signal received, but server **STILL ACCEPTING** connections.

**How:**
```rust
impl BeforeShutdown for DrainHandler {
    fn before_shutdown(&self, signal: ShutdownSignal) -> LifecycleFuture<'_> {
        Box::pin(async move {
            self.draining.store(true, Ordering::SeqCst);
            // Wait for in-flight requests to complete
            tokio::time::sleep(Duration::from_secs(2)).await;
            Ok(())
        })
    }
}
```

**Under the hood:** The shutdown signal (`SIGTERM`, `Ctrl-C`) is intercepted in `listen_with_shutdown()`. ALL `BeforeShutdown` callbacks run BEFORE the signal is passed to axum's `with_graceful_shutdown()`. This means the server is still accepting when your drain callback runs.

---

#### `OnApplicationShutdown` — Server stopped, clean up

**Why:** The server has stopped. Now clean up application-level resources.

**When:** After serving stops.

---

#### `OnModuleDestroy` — Per-module cleanup (reverse order)

**Why:** Close connections, flush buffers — in the OPPOSITE order of initialization.

**When:** During shutdown, reverse topological order.

---

#### `AfterShutdown` — Final last-chance cleanup

**Why:** After EVERYTHING is destroyed, one final operation (flush metrics, log duration).

**When:** After ALL `OnModuleDestroy` callbacks complete.

---

## Common patterns

### Seed data on first run

```rust
impl OnModuleInit for BlogService {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            if self.count_posts()? == 0 {
                self.create_post("Getting Started", "Welcome to Ironic!")?;
            }
            Ok(())
        })
    }
}
```

### Self health check after binding

```rust
impl OnServerReady for HealthChecker {
    fn on_server_ready(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let resp = reqwest::get("http://0.0.0.0:3000/health").await?;
            assert!(resp.status().is_success());
            tracing::info!("Health check passed");
            Ok(())
        })
    }
}
```

### Draining middleware

```rust
// In BeforeShutdown:
self.draining.store(true, Ordering::SeqCst);

// In your middleware:
if self.drain_flag.is_draining() {
    return Err(HttpError::service_unavailable("DRAINING", "server shutting down"));
}
```

## Error handling

| Failure during | What happens |
|---------------|-------------|
| `OnModuleInit` | App aborts. All initialized modules get `OnModuleDestroy` in reverse. |
| `OnApplicationBootstrap` | App aborts. Same reverse cleanup. |
| `OnServerReady` | App aborts. Same reverse cleanup. |
| `OnError` | Hook failure is logged. Original error still propagates. |
| `OnGuardDenied` | Hook failure is logged. 403 is still returned. |
| `BeforeShutdown` | Hook failure is logged. Shutdown continues. |
| `OnModuleDestroy` | Hook failure is logged. Remaining destroy hooks still run. |

**Design principle:** Startup failures are fatal (the app isn't ready). Runtime/shutdown failures are best-effort (log and continue).

## Next steps

→ [OnModuleConfigure details](./on-module-configure) — dynamic route registration  
→ [OnServerReady details](./on-server-ready) — orchestrator notification  
→ [Application Bootstrap](./application-bootstrap) — full guide for `OnApplicationBootstrap`
