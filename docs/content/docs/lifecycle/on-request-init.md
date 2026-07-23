---
title: OnRequestInit & OnRequestDestroy
description: Per-request scope lifecycle hooks — setup and teardown for each HTTP request.
---

# OnRequestInit & OnRequestDestroy

These hooks fire for **request-scoped** providers — they run when a request-scoped provider is first resolved and when the request scope ends.

## Use cases

- Initializing per-request auth context
- Allocating temporary resources (DB transactions, tracing spans)
- Flushing per-request metrics
- Closing temporary connections
- Logging request identifiers

## Signatures

```rust
pub trait OnRequestInit: Send + Sync + 'static {
    fn on_request_init(&self, request_id: &str) -> LifecycleFuture<'_>;
}

pub trait OnRequestDestroy: Send + Sync + 'static {
    fn on_request_destroy(&self) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnRequestInit, OnRequestDestroy, LifecycleError};

struct RequestContext {
    user_id: Option<String>,
    request_id: String,
    span: tracing::Span,
}

impl OnRequestInit for RequestContext {
    async fn on_request_init(&self, request_id: &str) -> Result<(), LifecycleError> {
        tracing::info!("Request started: {}", request_id);
        Ok(())
    }
}

impl OnRequestDestroy for RequestContext {
    async fn on_request_destroy(&self) -> Result<(), LifecycleError> {
        tracing::info!("Request ended: {}", self.request_id);
        Ok(())
    }
}
```

## Request scope lifecycle

```
Request arrives
    │
    ▼
OnRequestInit   — Provider is resolved for the first time
    │
    ▼
Route handler runs
    │
    ▼
Response sent
    │
    ▼
OnRequestDestroy — Provider scope is dropped
```

## Registration

```rust
ModuleDefinition::builder::<RequestContext>()
    .request_init()
    .request_destroy()
    .build()
```

## Transaction example

```rust
struct DbTransaction {
    pool: PgPool,
    tx: Option<Transaction<'static>>,
}

impl OnRequestInit for DbTransaction {
    async fn on_request_init(&self, _id: &str) -> Result<(), LifecycleError> {
        let tx = self.pool.begin().await
            .map_err(|e| LifecycleError::new(e.to_string()))?;
        // Store tx somewhere accessible
        Ok(())
    }
}

impl OnRequestDestroy for DbTransaction {
    async fn on_request_destroy(&self) -> Result<(), LifecycleError> {
        // Rollback if not explicitly committed
        Ok(())
    }
}
```

## Best practices

- Keep `OnRequestInit` fast — it blocks request processing
- Use `OnRequestDestroy` for cleanup that must happen even on errors
- Don't panic in either hook — errors are logged and the request continues
