---
title: Lifecycle & Pipeline
description: Understand how Ironic boots up, handles requests, and shuts down — the complete application lifecycle.
---

# Lifecycle & Pipeline

## What you'll learn

- The application lifecycle (startup, runtime, shutdown)
- The request pipeline order (middleware → guards → interceptors → handler)
- How to hook into lifecycle events

---

## Application lifecycle

```
┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────┐
│  START   │───►│ OnModuleInit │───►│ OnAppBootstrap│───►│ RUNNING  │
└──────────┘    └──────────────┘    └──────────────┘    └─────┬────┘
                                                               │
                                                       ┌───────▼──────┐
                                                       │  SHUTDOWN    │
                                                       │ OnModuleDestroy
                                                       │ OnAppShutdown│
                                                       └──────────────┘
```

### Lifecycle hooks

| Hook | When it runs | Use for |
|------|-------------|---------|
| `OnModuleInit` | After DI resolves, before server starts | DB migrations, cache warmup |
| `OnApplicationBootstrap` | After all modules init, before listening | Final validation, health checks |
| `OnModuleDestroy` | During shutdown, per module | Close connections, flush buffers |
| `OnApplicationShutdown` | Last step before process exit | Final cleanup, metrics flush |

### Example

```rust
use ironic::{OnModuleInit, OnModuleDestroy};

#[derive(Injectable)]
struct DatabaseService { pool: Arc<PgPool> }

impl OnModuleInit for DatabaseService {
    async fn on_module_init(&self) {
        println!("Running migrations...");
        sqlx::migrate!().run(&*self.pool).await.unwrap();
        println!("Migrations complete!");
    }
}

impl OnModuleDestroy for DatabaseService {
    async fn on_module_destroy(&self) {
        println!("Closing database pool...");
        self.pool.close().await;
    }
}
```

> **If init fails:** Ironic runs `OnModuleDestroy` in reverse order for all successfully initialized modules. No half-initialized state!

---

## Request pipeline

Every HTTP request flows through this pipeline:

```
HTTP Request
    │
    ▼
┌─────────────┐
│ Middleware   │ ← First: logging, auth, CORS
└──────┬──────┘
       ▼
┌─────────────┐
│   Guards     │ ← Auth check: Allow or Deny?
└──────┬──────┘
       ▼
┌─────────────┐
│ Interceptors │ ← Wrap: caching, serialization, timing
└──────┬──────┘
       ▼
┌─────────────┐
│ Extraction   │ ← Parse: path params, query, body
└──────┬──────┘
       ▼
┌─────────────┐
│   Pipes      │ ← Transform: validation, coercion
└──────┬──────┘
       ▼
┌─────────────┐
│   Handler    │ ← Your code! The actual route handler
└──────┬──────┘
       ▼
   Response
```

### When they fail (reverse unwinding)

```
Handler error ──► Exception filters (route → controller → global)
Guard denial  ──► Skips interceptors, extraction, handler entirely
Middleware error ──► Skips everything, returns error immediately
```

### Where to apply each

| Component | App-level | Controller-level | Route-level |
|-----------|-----------|-----------------|-------------|
| **Middleware** | ✅ `.middleware()` | ✅ `#[middleware]` | ✅ `#[middleware]` |
| **Guard** | ✅ `.guard()` | ✅ `#[guard]` | ✅ `#[guard]` |
| **Interceptor** | ✅ `.interceptor()` | ✅ `#[interceptor]` | ✅ `#[interceptor]` |
| **ExceptionFilter** | ✅ `.exception_filter()` | ✅ `#[exception]` | ✅ `.exception_filter()` |
| **Pipe** | ✅ `.pipe()` | ✅ | ✅ `.parameter_with_pipe()` |

---

## Health endpoint

Built-in, always available:

```rust
#[module(imports = [HealthModule])]
struct AppModule;
```

```bash
curl http://localhost:3000/health
# → {"status": "ok"}
```

No configuration needed — just import it.

---

## Try it yourself

1. Add `OnModuleInit` to a service that logs "Service ready!"
2. Add `OnModuleDestroy` that logs "Service shutting down..."
3. Start and stop the server — verify both messages appear
4. Create a custom Guard that checks a header and denies if missing

## What you learned

- [x] `OnModuleInit` / `OnModuleDestroy` run at module startup/shutdown
- [x] `OnApplicationBootstrap` / `OnApplicationShutdown` run app-wide
- [x] Request pipeline: Middleware → Guards → Interceptors → Extraction → Pipes → Handler
- [x] Failed init triggers reverse-order cleanup
- [x] `HealthModule` provides `GET /health` automatically
