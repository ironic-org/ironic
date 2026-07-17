---
title: OnRequestDestroy
description: Runs when a request scope ends — cleanup temp resources, flush metrics, close per-request connections.
---

# OnRequestDestroy

Runs when the request scope ends and the provider is about to be dropped. This is the **teardown pair** for `OnRequestInit`.

## When it fires

```
Handler returns response
    │
    ▼
OnRequestDestroy  ← YOU ARE HERE
    │
    ▼
Provider dropped, scope released
```

Runs **after** the response has been sent to the client. The hook runs even if the handler panicked (best-effort).

## The trait

```rust
pub trait OnRequestDestroy: Send + Sync + 'static {
    fn on_request_destroy(&self) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnRequestDestroy |
|---|---|
| Close a temporary file opened in `OnRequestInit` | Guaranteed cleanup |
| Commit or rollback a request-scoped DB transaction | Per-request TX boundary |
| Flush per-request metrics | Record handler duration, status code |
| Release rate-limit tokens | Time-based token return |

## Example — request-scoped transaction

```rust
struct RequestTxn {
    conn: Mutex<Option<Connection>>,
}

impl OnRequestInit for RequestTxn {
    fn on_request_init(&self, _request_id: &str) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let conn = pool.get().await?;
            conn.begin().await?;
            *self.conn.lock().unwrap() = Some(conn);
            Ok(())
        })
    }
}

impl OnRequestDestroy for RequestTxn {
    fn on_request_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            if let Some(conn) = self.conn.lock().unwrap().take() {
                let _ = conn.rollback().await;  // always rollback on scope end
            }
            Ok(())
        })
    }
}
```

## Guarantees

| Guarantee | Detail |
|---|---|
| Runs after response is sent | Client already received the response |
| Runs even on handler panic | Best-effort — errors are logged |
| Runs in reverse init order | Pair matching with OnRequestInit |
| Provider is still alive | All data is still accessible |
