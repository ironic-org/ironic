---
title: Circular Dependencies
description: Resolve circular dependencies with ForwardRef
---

# Circular Dependencies

Use `ForwardRef<T>` when two services depend on each other:

```rust
use ironic::{Injectable, ForwardRef};

#[derive(Injectable)]
struct ServiceA {
    b: ForwardRef<ServiceB>,
}

impl ServiceA {
    async fn do_something(&self) {
        let b = self.b.get().await;
        b.handle().await;
    }
}

#[derive(Injectable)]
struct ServiceB {
    a: Arc<ServiceA>,
}
```

`ForwardRef<T>` resolves lazily — the DI container fills it after all
singletons are constructed. The `.get()` method awaits the value if it
hasn't been populated yet.

## With `#[forward_ref]`

The optional `#[forward_ref]` annotation makes the intent explicit:

```rust
#[derive(Injectable)]
struct ServiceA {
    #[forward_ref]
    b: ForwardRef<ServiceB>,
}
```

## How It Works

1. Service A is constructed with an empty `ForwardRef<ServiceB>`
2. Service B is constructed with the fully-resolved `Arc<ServiceA>`
3. After all singletons are built, the container fills the `ForwardRef`
4. Service A can now access Service B via `.get()`
