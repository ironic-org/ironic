---
title: Mocking Dependencies
description: Swap real services for test doubles with provider overrides, and test single modules in isolation with TestModule.
---

# Mocking Dependencies

## What you'll learn

- Replace real services with mocks using `override_provider`
- Test a single module in isolation with `TestModule`
- Build containers with mocked dependencies directly

---

## Mocking with provider overrides

The real power of DI: swap real services for test doubles:

```rust
use ironic::{ContainerBuilder, ProviderDefinition, TestApplication};
use ironic::prelude::*;

#[tokio::test]
async fn uses_mock_service() {
    // Create a mock service
    let mock_service = MockUserService {
        users: vec![User { id: 1, name: "Test User".into() }],
    };

    // Build a container with the mock instead of the real service
    let mut container = ContainerBuilder::new();
    container
        .register(ProviderDefinition::value(mock_service))
        .unwrap();

    // Override the UserService to use our mock
    let app = TestApplication::builder()
        .module::<AppModule>()      // ← Real app module
        .override_provider::<UserService>(mock_service)  // ← But swap the service
        .build()
        .await
        .unwrap();

    app.get("/users").send().await.assert_json(&vec![User { id: 1, name: "Test User".into() }]);
    app.shutdown().await.unwrap();
}
```

> Overrides must be set before calling `.build()` — the container is immutable after build. For the provider system behind this, see [Dependency Management](/docs/core/dependency-management).

## Testing modules in isolation

Test a single module without the full app:

```rust
use ironic::TestModule;

#[tokio::test]
async fn test_products_module_alone() {
    let module = TestModule::new::<ProductsModule>()
        .await
        .unwrap();

    module.get("/products").send().await.assert_status(200);
    module.shutdown().await.unwrap();
}
```

`TestModule` builds only the providers and controllers that the module declares — external dependencies must either be overridden or provided by the module itself.

## When to mock

| Situation | Approach |
|-----------|----------|
| Slow external service (email, SMS, payment) | `override_provider` with a stub |
| Flaky dependency in CI | `override_provider` with a deterministic fake |
| Testing a single controller | `TestModule::new::<ControllerModule>()` |
| End-to-end flow across modules | `TestApplication` with targeted overrides |

## What you learned

- [x] `override_provider::<T>(val)` swaps a service for a test double
- [x] `TestModule::new::<Module>()` tests a single module in isolation
- [x] Overrides must be set before `.build()`
- [x] Prefer `TestModule` for fast unit-level integration tests
