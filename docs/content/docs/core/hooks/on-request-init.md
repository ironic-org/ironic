---
title: OnRequestInit
description: Runs when a request-scoped provider is created — per-request setup, auth context, temp resources.
---

# OnRequestInit

Runs when a request-scoped provider is first resolved within a new HTTP request. Think of it as the **constructor for per-request state**.

## When it fires

```
HTTP Request arrives
    │
    ├─ Route matched, handler identified
    ├─ Request-scoped providers resolved
    │   └─ OnRequestInit  ← YOU ARE HERE
    ▼
Middleware → Guards → Handler
```

The `request_id` parameter is the framework-generated unique ID from `RequestTracing`.

## The trait

```rust
pub trait OnRequestInit: Send + Sync + 'static {
    fn on_request_init(&self, request_id: &str) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnRequestInit |
|---|---|
| Initialize per-request auth context | Extract user from token, put in extensions |
| Allocate a temporary file for upload processing | Scope-bound resource cleanup |
| Log request start for distributed tracing | Request ID + start timestamp |
| Set up a request-scoped database transaction | Begin TX, commit/rollback in destroy |

## Example — request logger

```rust
use std::time::Instant;

#[derive(Injectable)]
pub struct RequestTracker {
    start: Option<Instant>,
    request_id: Option<String>,
}

impl OnRequestInit for RequestTracker {
    fn on_request_init(&self, request_id: &str) -> LifecycleFuture<'_> {
        // Note: use interior mutability (Mutex) for per-request state
        Box::pin(async move {
            tracing::debug!(request_id, "request started");
            Ok(())
        })
    }
}

impl OnRequestDestroy for RequestTracker {
    fn on_request_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            tracing::debug!("request completed");
            Ok(())
        })
    }
}
```

## Request lifecycle pair

`OnRequestInit` and `OnRequestDestroy` are designed as a **pair**:

```
OnRequestInit  ← setup, allocate, begin
     │
  [handler runs]
     │
OnRequestDestroy  ← teardown, deallocate, end
```

The destroy hook is guaranteed to run even if the handler panics (best-effort).

## Important: mutability

Both `on_request_init` and `on_request_destroy` take `&self` (immutable reference). Since providers are shared via `Arc`, use **interior mutability** (`Mutex`, `RwLock`, `Atomic*`) for per-request state:

```rust
struct RequestTracker {
    start: Mutex<Option<Instant>>,  // interior mutability
}
```
