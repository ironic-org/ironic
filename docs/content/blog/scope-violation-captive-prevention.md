---
title: "ScopeViolation — how the resolver prevents singletons from capturing request-scoped state"
description: "Ironic's DI resolver enforces encapsulation boundaries at runtime: if a singleton factory attempts to resolve a request-scoped dependency, the resolver returns a ScopeViolation error instead of silently creating a captive instance."
date: "2026-07-15"
author: "Ironic Team"
---

# ScopeViolation — how the resolver prevents singletons from capturing request-scoped state

Every DI framework with hierarchical scopes faces the same correctness hazard: the captive dependency. A singleton (lives forever) depends on a request-scoped provider (lives for one HTTP request). If the container silently satisfies this dependency, the singleton captures a reference to state that becomes stale — or worse, a dangling reference after the request ends. The server doesn't crash immediately; it serves wrong data for weeks until someone notices.

Most frameworks handle this through documentation or convention. Ironic handles it at the resolver level — the container refuses to construct a singleton that wants request-scoped state and returns a dedicated error variant that tells you exactly what went wrong.

---

## The classic anti-pattern

Here's the setup that causes problems in most DI frameworks:

```rust
struct RequestContext { user_id: String, tenant: String }
// Registered as Scope::Request — one instance per HTTP request

struct EmailService { request_context: Arc<RequestContext> }
// Registered as Scope::Singleton — one instance for the lifetime of the app
```

The intent is understandable: `EmailService` needs the current user's identity to send emails. But a singleton is constructed once. The first request to hit the server determines `user_id` and `tenant`, and every subsequent request — from entirely different users — gets the same captured values. If you're lucky, this manifests as a bug report about emails going to the wrong person. If you're not, it silently corrupts audit logs for months.

In frameworks like NestJS, you can declare `@Injectable({ scope: Scope.REQUEST })` to make a provider transient, but there's no compile-time or runtime check that prevents a singleton-scoped provider from injecting a request-scoped one. The scope cascading happens implicitly, and the captive dependency is invisible until production.

---

## How Ironic detects it: the `request_allowed` flag

Every `Resolver` in Ironic carries a boolean field called `request_allowed`:

```rust
pub struct Resolver {
    container: Container,
    path: Arc<[ProviderKey]>,
    request_cache: Option<Arc<RequestCache>>,
    request_allowed: bool,
}
```

This flag controls whether request-scoped resolution is permitted at the current point in the dependency graph. When you create a top-level resolver via `Container::request_scope()`, `request_allowed` starts as `true` — you're at the request boundary and resolving request-scoped providers is legitimate.

The critical logic lives in `resolve_erased`, the internal method that handles every provider resolution. When a registration's scope is `Scope::Request`, two checks fire:

```rust
if registration.definition.scope == Scope::Request {
    if self.request_cache.is_none() {
        return Err(ResolveError::RequestScopeRequired { key, path });
    }
    if !self.request_allowed {
        return Err(ResolveError::ScopeViolation { key, path });
    }
}
```

The first check ensures we're inside a request scope at all (no request cache means no request is active). The second check — the one that catches captive dependencies — ensures that even within a request scope, we're not trying to resolve request-scoped state from a position that shouldn't have access to it.

---

## How `request_allowed` propagates downward

The key insight is in how child resolvers are constructed. When the resolver builds the child resolver for a dependency's factory, it sets `request_allowed` based on the dependency's own scope:

```rust
let child = Self {
    container: self.container.clone(),
    path: path.clone().into(),
    request_cache: self.request_cache.clone(),
    request_allowed: self.request_allowed
        && registration.definition.scope != Scope::Singleton,
};
```

Read this carefully: `request_allowed` in the child resolver is `true` only if it was already `true` in the parent *and* the registration currently being resolved is not a `Scope::Singleton`. The moment resolution enters a singleton's factory, every descendant resolver gets `request_allowed: false`. Request-scoped access is severed at the singleton boundary.

This means the scope violation propagates through arbitrary depths. If a singleton depends on a transient provider, and that transient provider depends on a request-scoped provider, the violation is caught at the transient's attempt to resolve the request-scoped dependency — because the transient's resolver inherited `request_allowed: false` from the singleton above it.

---

## The error message

When the violation fires, the error is explicit:

```
IRONIC_DI_SCOPE_VIOLATION: singleton construction cannot resolve request provider `{key}`
```

The error includes the full `ProviderKey` of the offending request-scoped dependency and the dependency `path` — a chain of provider keys showing the exact resolution trace from the entry point down to the violation. This makes debugging straightforward: you can see which singleton depends on which transient, which ultimately depends on which request-scoped provider.

---

## The dedicated test

The crate includes a minimal reproduction that verifies the detection works:

```rust
#[tokio::test]
async fn singleton_cannot_capture_request_scoped_state() {
    struct RequestValue;
    struct Singleton;

    let mut builder = ContainerBuilder::new();
    builder
        .register(ProviderDefinition::constructor(
            Scope::Request, Vec::new(), |_resolver| Ok(RequestValue),
        )).unwrap()
        .register(ProviderDefinition::factory(
            Scope::Singleton,
            vec![Dependency::required::<RequestValue>()],
            |resolver| async move {
                resolver.resolve::<RequestValue>().await?;
                Ok(Singleton)
            },
        )).unwrap();

    assert!(matches!(
        builder.build().request_scope().resolve::<Singleton>().await,
        Err(ResolveError::ScopeViolation { .. })
    ));
}
```

The test registers `RequestValue` as request-scoped and `Singleton` with an explicit dependency on `RequestValue`. It then tries to resolve `Singleton` within a request scope. The assertion confirms that the resolution fails with `ScopeViolation` — not a panic, not a hang, not a silently-constructed wrong instance.

---

## Runtime safety without lifetime annotations

Rust's type system could theoretically prevent this at compile time with complex lifetime annotations — requiring every singleton to prove it outlives every request-scoped dependency. But that would infect every type signature in the application with lifetime parameters, many of which would propagate up to user code. The ergonomic cost isn't worth the compile-time guarantee.

Instead, Ironic chooses runtime detection: the violation is caught at first access, inside a test or at application startup (if an eager singleton depends on request-scoped state). The error is immediate, deterministic, and points directly at the offending provider. A single test exercising your singleton construction path is sufficient to catch every captive dependency in the graph.

This is the same trade-off that languages with gradual typing make: you accept a runtime check in exchange for not contaminating your entire API surface with proof obligations. The difference is that the check runs the moment the container resolves the faulty provider — not three weeks later when a customer reports seeing someone else's data.
