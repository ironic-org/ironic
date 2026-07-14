---
title: Request-Scoped Providers
description: Create services that live for one HTTP request — per-request user context, tracing IDs, and request metadata.
---

# Request-Scoped Providers

## What you'll learn

- Create services that are created fresh for every HTTP request
- Inject request-specific data (user ID, trace ID)
- Understand when to use request scope vs singleton

---

## Singleton vs Request scope

| Scope | Lives for | Example |
|-------|-----------|---------|
| **Singleton** (default) | Entire app lifetime | Database pool, config, caches |
| **Request** | One HTTP request | Current user, request ID, per-request counters |

## Creating a request-scoped provider

```rust
#[derive(Injectable)]
#[injectable(scope = "request")]
pub struct RequestContext {
    pub user_id: Option<u64>,
    pub trace_id: String,
    pub started_at: std::time::Instant,
}
```

A new `RequestContext` is created for every request and destroyed after the response is sent.

## Using it

```rust
#[derive(Injectable)]
pub struct AuditService {
    ctx: std::sync::Arc<RequestContext>,  // ← Fresh per request
    db: std::sync::Arc<PgPool>,           // ← Singleton, shared
}

impl AuditService {
    pub fn log(&self, action: &str) {
        println!(
            "[user: {:?}] [trace: {}] {action}",
            self.ctx.user_id, self.ctx.trace_id
        );
    }
}
```

> **Key rule:** A request-scoped provider can depend on singletons, but a singleton can **never** depend on a request-scoped provider. The compiler enforces this!

## What you learned

- [x] `#[injectable(scope = "request")]` creates per-request instances
- [x] Request-scoped = created new for every HTTP request
- [x] Singletons CANNOT depend on request-scoped providers
- [x] Use for: user context, request metadata, per-request counters
