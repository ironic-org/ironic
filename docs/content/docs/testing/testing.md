---
title: Testing
description: Write fast, reliable tests for your Ironic application — no sockets, no Docker, instant feedback.
---

# Testing

## What you'll learn

- Write integration tests that run in milliseconds (no real HTTP server)
- Mock and override dependencies for isolated testing
- Assert HTTP responses with fluent builders
- Test your entire API without starting a server

> **Why this matters:** Ironic tests run **in-process** — no network sockets, no port conflicts, no Docker. A full API test takes microseconds, not seconds.

## Quick start

```rust
use ironic::TestApplication;
use ironic::prelude::*;

#[tokio::test]
async fn get_user_returns_200() {
    // 1. Create a test app (no server, instant startup)
    let app = TestApplication::new::<AppModule>()
        .await
        .expect("test app should start");

    // 2. Send a request and assert the response
    app.get("/users/42")
        .send()
        .await
        .assert_status(200);

    // 3. Clean up
    app.shutdown().await.unwrap();
}
```

## TestModule vs TestApplication vs TestModuleBuilder

| Type | Scope | Best for | Startup cost |
|------|-------|----------|-------------|
| `TestApplication::new::<AppModule>()` | Full app | End-to-end integration tests | Full DI graph |
| `TestApplication::builder()` | Full app with overrides | Mocking specific services | Full DI graph |
| `TestModule::new::<Module>()` | Single module | Testing one controller in isolation | Minimal |
| `TestModuleBuilder::new()` | Single module with overrides | Mocking dependencies of one module | Minimal |

> Prefer `TestModule` for unit-level integration tests (faster). Use `TestApplication` for end-to-end flows that span multiple modules.

## Test isolation

Every test starts with a **fresh DI container**. Ironic does not share state between tests:

```rust
#[tokio::test]
async fn test_a() {
    let app = TestApplication::new::<AppModule>().await.unwrap();
    // Creates a new container, all services instantiated fresh
    app.shutdown().await.unwrap();
} // Container is dropped — no leakage

#[tokio::test]
async fn test_b() {
    let app = TestApplication::new::<AppModule>().await.unwrap();
    // Completely independent from test_a — no state leakage
    app.shutdown().await.unwrap();
}
```

Key points about isolation:

- Each `TestApplication` / `TestModule` gets its own `Container` instance
- No shared singletons between tests
- Providers registered as **transient** are recreated per request within the same test
- Always call `.shutdown()` to ensure background tasks and connections are cleaned up
- Tokio's test runtime runs each `#[tokio::test]` in a separate task context

## Going deeper

The testing section is split into focused guides:

| Guide | Covers |
|-------|--------|
| [TestApplication & assertions](/docs/testing/test-application) | Builder API, request builders, `TestResponse` assertion API, fluent assertions |
| [Mocking dependencies](/docs/testing/mocking) | Swapping real services for test doubles, `TestModule` in isolation |
| [CI setup](/docs/testing/ci) | GitHub Actions, complete test example, integration test hygiene |

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Forgetting `await` on `.send()` | All test methods return futures — must be awaited |
| Not shutting down | Always call `.shutdown().await.unwrap()` to clean up |
| Mocking wrong type | Make sure the mock type matches the service being overridden |
| Tests not `#[cfg(test)]` gated | Wrap tests in `#[cfg(test)] mod tests { ... }` so they don't compile in release |
| Reusing a shut-down app | After `.shutdown()`, the container is destroyed. Create a new `TestApplication` for each test |
| Overriding after `.build()` | All overrides must be set before calling `.build()` — the container is immutable after build |

## What you learned

- [x] `TestApplication` runs full integration tests without a real server
- [x] Tests run in microseconds — no network overhead
- [x] Each test gets a fresh container — no state leakage between tests
- [x] `TestModule` tests a single module in isolation
