---
title: Request-scoped providers
description: Create dependency-injection values once per HTTP request.
---

# Request-scoped providers

Use request scope for values that must be isolated between requests but shared by every controller,
service, and repository participating in one request:

```rust,ignore
ProviderDefinition::factory(
    Scope::Request,
    vec![Dependency::required::<DatabasePool>()],
    |resolver| async move {
        let pool = resolver.resolve::<DatabasePool>().await?;
        Ok(UnitOfWork::begin(&pool).await?)
    },
)
```

With macros, select it on an injectable type:

```rust,ignore
#[derive(Injectable)]
#[injectable(scope = "request")]
struct CurrentRequest { /* ... */ }
```

Ironic creates a `RequestScope` before the request pipeline starts. The first resolution constructs
the provider; concurrent and later resolutions in that request receive the same `Arc`. A new HTTP
request receives a different value.

Singleton providers cannot depend on request-scoped providers because that would retain one
request's state globally. Ironic reports `IRONIC_DI_SCOPE_VIOLATION` if a singleton factory tries
to do so. Resolving a request provider directly from `Container` reports
`IRONIC_DI_REQUEST_SCOPE_REQUIRED`; use `container.request_scope()` for non-HTTP jobs and tests.
