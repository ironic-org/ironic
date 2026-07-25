---
title: Circular Dependencies
description: What they are, why they happen, and how to resolve them with ForwardRef
---

# Circular Dependencies

A **circular dependency** happens when two or more services depend on each other
directly or indirectly. The DI container cannot construct them because each one
requires the other to exist first.

## The Problem

Consider two services that need each other:

```
ServiceA ──depends on──▶ ServiceB
    ▲                       │
    │                       │
    └───depends on──────────┘
```

```rust
struct ServiceA {
    b: Arc<ServiceB>,  // ServiceA needs ServiceB
}

struct ServiceB {
    a: Arc<ServiceA>,  // ServiceB needs ServiceA
}
```

When the DI container tries to construct `ServiceA`:
1. It sees `ServiceA` needs `ServiceB`
2. It tries to construct `ServiceB`
3. `ServiceB` needs `ServiceA` — which is **still being constructed**
4. **Deadlock** — neither can be built

Without `ForwardRef`, the container returns a `CircularDependency` error:

```
RF_DI_CIRCULAR_DEPENDENCY: resolving `ServiceA` would create a cycle
```

## The Solution: `ForwardRef<T>`

`ForwardRef<T>` breaks the cycle by allowing **lazy resolution**.
One side of the cycle stores a placeholder that gets filled after both
services are constructed.

```rust
use ironic::{Injectable, ForwardRef};

// ServiceA stores a lazy reference to ServiceB
#[derive(Injectable)]
struct ServiceA {
    b: ForwardRef<ServiceB>,
}

impl ServiceA {
    async fn do_something(&self) {
        // b.get() awaits until the container fills the reference
        let b = self.b.get().await;
        b.handle().await;
    }
}

// ServiceB stores a normal resolved reference to ServiceA
#[derive(Injectable)]
struct ServiceB {
    a: Arc<ServiceA>,
}
```

## How It Works Step by Step

```
Timeline:
─────┬──────────────────────────────────────────
     │
  1  │  Container starts building singletons
     │
  2  │  Constructing ServiceA...
     │  ┌─ Sees ForwardRef<ServiceB>
     │  ├─ Creates EMPTY placeholder ref
     │  ├─ Registers pending resolution callback
     │  └─ Returns ServiceA (with empty ref)
     │
  3  │  Constructing ServiceB...
     │  └─ Resolves Arc<ServiceA> from singleton cache
     │     (ServiceA is already fully constructed!)
     │
  4  │  Container runs pending callbacks
     │  └─ Fills ServiceA's ForwardRef<ServiceB>
     │     with the resolved ServiceB
     │
  5  │  ServiceA.b.get() now returns the real ServiceB ✓
     │
─────┴──────────────────────────────────────────
```

### Why This Works

The key insight: **`ForwardRef<T>` does NOT call `resolver.resolve::<T>()`**
during construction. Instead, it stores an empty `OnceLock` and registers
a deferred resolution. This means:

- `ServiceA` is constructed **instantly** without waiting for `ServiceB`
- `ServiceB` is constructed normally (it gets `Arc<ServiceA>` from the cache)
- After **all** singletons are built, the container fills all `ForwardRef` slots

## When to Use `ForwardRef`

✅ **Use it when:**
- Two services genuinely need each other to function
- The circular dependency is inherent to your domain model
- Both services are singletons

❌ **Avoid it when:**
- You can restructure to remove the cycle
- One direction is truly optional (use `Option<ForwardRef<T>>` or optional deps)
- The cycle spans more than 2-3 services (refactor instead)

## With `#[forward_ref]` Annotation

The optional `#[forward_ref]` attribute makes the intent explicit and helps
with code readability:

```rust
#[derive(Injectable)]
struct ServiceA {
    #[forward_ref]
    b: ForwardRef<ServiceB>,
}
```

This is purely documentation — the `Injectable` derive already detects
`ForwardRef<T>` by type. The annotation helps readers understand that
this field is intentionally lazy.

## Real-World Example

A common real-world circular dependency: **NotificationService** ↔ **UserService**

```rust
use ironic::{Injectable, ForwardRef};

// ── UserService ──────────────────────────────────
#[derive(Injectable)]
struct UserService {
    // Sends notifications when users register
    notifications: ForwardRef<NotificationService>,
}

impl UserService {
    async fn register(&self, name: &str) -> User {
        let user = User::new(name);
        // Notify asynchronously
        let notifier = self.notifications.get().await;
        notifier.send_welcome(&user).await;
        user
    }
}

// ── NotificationService ──────────────────────────
#[derive(Injectable)]
struct NotificationService {
    // Looks up user preferences for notification routing
    users: Arc<UserService>,
}

impl NotificationService {
    async fn send_welcome(&self, user: &User) {
        let prefs = self.users.get_preferences(user.id).await;
        if prefs.email_notifications {
            println!("Sending welcome email to {}", user.email);
        }
    }
}
```

## Common Mistakes

| Mistake | Why It Fails | Fix |
|---------|-------------|-----|
| Using `Arc<T>` on both sides | Container detects cycle at resolve time | Use `ForwardRef<T>` on one side |
| Calling `.get()` before resolution | `.get()` awaits until populated (safe) | Ensure container resolves forward refs (automatic) |
| `ForwardRef` on non-singleton | Transient/request-scoped don't share state | Only use between singletons |
| Chain of 3+ circular deps | Design issue — too tightly coupled | Extract shared logic to a third service |

## Performance

`ForwardRef<T>` has minimal overhead:

- **Construction:** An empty `OnceLock` allocation
- **Resolution:** One `Arc::clone()` after all singletons are built
- **Access:** `ForwardRef::get()` loops with `yield_now()` until populated
  (typically resolves within microseconds during startup)

## API Reference

| Method | Description |
|--------|-------------|
| `ForwardRef::new()` | Creates an empty forward reference |
| `ForwardRef::get().await` | Resolves the reference, awaiting if needed |
| `ForwardRef::shared_inner()` | Returns the inner `OnceLock` for pre-population |
