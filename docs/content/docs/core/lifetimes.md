---
title: Service Lifetimes
description: Control how and when services are created — Singleton, Transient, and Request-scoped providers with eager initialization.
---

# Service Lifetimes

## What you'll learn

- The three provider scopes — Singleton, Transient, and Request
- How to set a scope via `#[injectable(scope = "...")]`
- How Request-scoped providers work inside HTTP handlers
- When to use eager initialization (`#[injectable(eager)]`)
- How scope violations are detected at runtime
- How to define scopes manually with `ProviderDefinition`

---

## The three scopes

Ironic has three lifetime policies for providers:

| Scope | Instances | Created | Use case |
|-------|-----------|---------|----------|
| **Singleton** *(default)* | One per container | On first resolution | Database pools, config, caches — anything stateful and shared |
| **Transient** | New every resolution | On every `resolve()` call | Stateless helpers, ID generators, lightweight utilities |
| **Request** | One per HTTP request | On first resolution within a request | User context, request tracing, per-request state |

```rust
pub enum Scope {
    Singleton,  // Default — once per container (backed by OnceCell)
    Transient,  // Fresh instance on every resolve
    Request,    // One instance shared within a single HTTP request
}
```

---

## Setting scope via `#[injectable]`

Use the `scope` option on the `#[injectable(...)]` attribute:

```rust
#[derive(Injectable)]
#[injectable(scope = "transient")]
pub struct IdGenerator;

#[derive(Injectable)]
#[injectable(scope = "request")]
pub struct RequestContext {
    pub user_id: Option<u64>,
}
```

The default is `"singleton"` — if you omit the `scope` option, your service is a singleton.

---

## Request-scoped providers deep dive

Request-scoped providers are created once per HTTP request and discarded afterward. They are **not** resolvable from a bare `Container::resolve()`:

```rust
// Compiles but panics at runtime with RequestScopeRequired
container.resolve::<RequestContext>().await;
// Use the request scope instead:
let scope = container.request_scope();
let ctx = scope.resolve::<RequestContext>().await.unwrap();
```

**In HTTP handlers** this is automatic — the framework creates a `RequestScope`, inserts it into `RequestContext` extensions, and all route-handler injection goes through it.

### Scope violation rules

Singletons **cannot** depend on request-scoped providers. Since singletons outlive any individual request, capturing request-scoped state would be unsound:

```rust
#[derive(Injectable)]
pub struct CacheWarmer {
    ctx: Arc<RequestContext>,  // Compiles — but...
}

// At runtime: "ScopeViolation: singleton CacheWarmer
//               depends on request-scoped RequestContext"
```

The DI container rejects this at resolution time with a `ResolveError::ScopeViolation` that includes the full dependency path.

---

## Eager initialization

By default, providers are lazily constructed on first use. Add `eager` to force construction during application bootstrap:

```rust
#[derive(Injectable)]
#[injectable(eager)]
pub struct DatabasePool {
    pool: PgPool,
}
```

> **Why eager?** A connection failure at startup gives you a clear error immediately. A connection failure on the 1000th request at 3 AM is much worse.

---

## Manual `ProviderDefinition`

When you need full control (e.g. for third-party types you can't derive on), register a provider manually:

```rust
use ironic::di::{ProviderDefinition, Scope, Dependency};

ProviderDefinition::constructor(
    Scope::Singleton,
    vec![],
    |_| Ok(MyService::new()),
)
```

Use `ProviderDefinition::factory(...)` for async construction and `ProviderDefinition::value(...)` to inject a pre-built instance.

```rust
ProviderDefinition::value(my_prebuilt_config)  // Singleton, no deps
```

---

## Scope decision guide

| Situation | Recommended scope |
|-----------|-------------------|
| Shared state (DB pool, cache, config) | **Singleton** |
| Stateless computation (hashing, formatting) | **Transient** |
| Per-request data (user session, trace ID) | **Request** |
| Service that validates at startup (DB health) | **Singleton + eager** |

---

## Common pitfalls

| Pitfall | What happens | Fix |
|---------|-------------|-----|
| Singleton → Request dependency | `ScopeViolation` at runtime | Promote to Transient, or pass data via method args |
| Using `Request` scope outside HTTP | `RequestScopeRequired` error | Call `container.request_scope()` first |
| Not marking `eager` on a DB pool | Connection error surfaces on first user request | Add `#[injectable(eager)]` |
| Confusing Transient with "per call" | A new instance is created every resolve, even within the same request | Use Request scope for per-request state |

## What you learned

- [x] `Singleton` (default) creates one shared instance per container
- [x] `Transient` creates a new instance on every resolution
- [x] `Request` creates one instance per HTTP request, auto-managed by `RequestScope`
- [x] Singletons cannot depend on request-scoped providers (`ScopeViolation`)
- [x] `#[injectable(eager)]` forces construction at bootstrap
- [x] `ProviderDefinition` gives you manual control over scope and factory
