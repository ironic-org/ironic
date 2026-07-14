---
title: Dependency Management
description: Master Ironic's dependency injection — optional deps, eager initialization, circular dependency detection.
---

# Dependency Management

## What you'll learn

- Mark dependencies as optional (fail gracefully if missing)
- Force services to initialize eagerly at startup
- Understand how Ironic detects circular dependencies

---

## Optional dependencies

Sometimes a dependency might not be registered. Use optional resolution:

```rust
use ironic::ModuleRef;

#[derive(Injectable)]
struct AnalyticsService {
    // Will be None if CacheService isn't registered
    cache: Option<std::sync::Arc<CacheService>>,
}

impl AnalyticsService {
    pub fn track(&self, event: &str) {
        if let Some(cache) = &self.cache {
            cache.set(event, 1);
        }
        // Silently skip caching if not configured
    }
}
```

## Eager initialization

By default, services are created on first use. Force immediate creation:

```rust
#[derive(Injectable)]
#[injectable(eager)]
pub struct DatabasePool {
    // Created at startup, not on first request
    // Catches connection errors early!
}
```

> **Use eager for:** Database pools, external service connections. Catch misconfigurations at startup, not at 3 AM.

## Circular dependency detection

Ironic catches circular dependencies at startup:

```rust
struct A { b: Arc<B> }    // A depends on B
struct B { a: Arc<A> }    // B depends on A
                          // → Error: "Circular dependency detected: A → B → A"
```

The error message shows the full dependency chain so you can fix it.

## What you learned

- [x] Optional deps return `None` when not registered
- [x] `#[injectable(eager)]` forces startup initialization
- [x] Circular dependencies are detected and reported clearly
