---
title: "OnceCell-based singletons — cancellation-safe initialization and retry"
description: "How Ironic uses tokio::sync::OnceCell to build singleton providers that survive cancellation, retry on failure, and never poison shared state."
date: "2026-07-15"
author: "Ironic Team"
---

# OnceCell-based singletons — cancellation-safe initialization and retry

Most DI containers implement singletons with a `Mutex<Option<T>>`. Store the value on first access, lock and check on every subsequent access. It works — until initialization fails. A poisoned mutex or a stale `None` after an error requires hand-rolled retry logic, and cancellation safety is an afterthought. Ironic sidesteps all of this by building singletons on top of `tokio::sync::OnceCell`.

The building block is `OnceCell::get_or_try_init()`, exposed through a concise dispatch path in `crates/ironic-di/src/lib.rs:639-644`:

```rust
match registration.definition.scope {
    Scope::Singleton => registration
        .singleton
        .get_or_try_init(construct)
        .await
        .cloned(),
    // ...
}
```

Each `Registration` struct holds a `OnceCell<ProviderValue>` (`lib.rs:410`). The container stores an `Arc<Registration>` per key, so all concurrent callers share the same cell.

## The semantics: initialize once, retry on error

`get_or_try_init` accepts an async closure that returns `Result<T, E>`. It behaves differently from a simple `HashMap`:

- **If the factory succeeds**, the value is stored and every subsequent call — concurrent or sequential — receives the same `Arc<T>` without invoking the factory again.
- **If the factory returns `Err`**, the `OnceCell` remains empty. It does *not* store the error. The next caller will retry the full initialization path.
- **If the factory is cancelled** (e.g. `tokio::select!` drops the future, or `task.abort()` is called), the cell also remains empty, and a subsequent caller retries.

This is the critical difference from `Mutex<Option<T>>`. With a mutex, you must decide: do I cache the error? If yes, you need a separate retry path. If no, all errors are transient and you lose the ability to distinguish "never initialized" from "initialized but stale." OnceCell's atomic state machine handles both states without any bookkeeping.

## Failed initialization — retry in action

The test at line 822 demonstrates the chain:

```rust
let calls = Arc::new(AtomicUsize::new(0));
// factory: first attempt returns Err, second returns Ok
// ...
assert!(container.resolve::<Repository>().await.is_err());
assert!(container.resolve::<Repository>().await.is_ok());
assert_eq!(calls.load(Ordering::SeqCst), 2);
```

The first `resolve()` hits the factory, which increments the counter from 0 to 1 and returns `ResolveError::factory::<Repository>("not ready")`. The `OnceCell` is still empty. The second `resolve()` invokes the factory again (counter 1→2), this time succeeding. The assertion confirms the factory was called exactly twice — once for the error, once for the success.

There is no stale error state. The consumer doesn't need to distinguish "initialization errored" from "never tried." Both states are the same from the caller's perspective: call `resolve()`, and the framework handles retry.

## Cancelled initialization — the poison-free contract

The test at line 851 is more interesting. A factory function calls `std::future::pending().await` — it never resolves. The test spawns a task to call `resolve()`, waits for the factory to start (via `Notify`), then calls `first.abort()`:

```rust
let first = tokio::spawn({
    let container = container.clone();
    async move { container.resolve::<Repository>().await }
});
started.notified().await;
first.abort();
assert!(first.await.unwrap_err().is_cancelled());
assert!(container.resolve::<Repository>().await.is_ok());
assert_eq!(attempts.load(Ordering::SeqCst), 2);
```

After the spawned task is aborted, `OnceCell` is still empty. A subsequent `resolve()` calls the factory on its second attempt (counter 1→2), which now returns `Ok`. The singleton is alive and the original cancellation did not poison anything.

This matters because cancellation is pervasive in async Rust. `tokio::select!` drops futures silently. Connection timeouts abort in-flight work. Without OnceCell's semantics, a cancelled factory could leave a mutex locked or a half-written `Option` — both of which would permanently block the singleton.

## Why OnceCell over Mutex\<Option\<T\>\>

| Property | `Mutex<Option<T>>` | `OnceCell` |
|---|---|---|
| Error retry | Manual: store a sentinel, implement fallback | Automatic: empty cell = retry |
| Cancellation after start | Lock may be poisoned or held indefinitely | No lock held; cell stays empty |
| Concurrent initialization | Serialized through the mutex | `get_or_try_init` handles multiple callers — only one runs the factory |
| Stale error state | Must implement expiry or explicit reset | No error is stored; cell is empty |

The `Mutex<Option<T>>` approach forces you to recreate the semantics OnceCell already provides: serialized initialization, error-agnostic retry, and cancellation safety. OnceCell bakes them into the type system.

## A concrete trace

Two concurrent HTTP requests both need a database connection pool. Both call `container.resolve::<Pool>()`:

1. Request A arrives first. `OnceCell::get_or_try_init()` begins executing the factory — connecting to the database, running migrations, warming connections.
2. Request B arrives while A's factory is still running. `get_or_try_init` sees that initialization is in-flight and waits on the same future. It does *not* invoke the factory again.
3. Request A's factory completes successfully. The `OnceCell` stores the `Arc<Pool>`.
4. Both requests receive cloned `Arc`s to the same pool.

If step 3 had failed instead, both waiters would observe the failure. Request B, arriving later, would trigger a new factory call — transparent retry with no consumer involvement. If Request A were cancelled mid-initialization, Request B would see an empty cell and start a fresh attempt.

This is what makes Ironic's singleton semantics robust in production: the framework treats initialization failures as transient infrastructure events, not as permanent broken state. OnceCell gives you that contract with zero lines of retry logic.
