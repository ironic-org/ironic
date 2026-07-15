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

## TestApplication builder API

For advanced scenarios, use the builder to assemble test apps with precise control:

```rust
use ironic::{TestApplication, TestApplicationBuilder};

let app = TestApplication::builder()
    .module::<AppModule>()                 // Register a module by type
    .module::<AuthModule>()                // Add another module
    .override_provider::<UserService>(mock_service)  // Swap a dependency
    .override_provider::<CacheService>(mock_cache)   // Swap another
    .build()                               // Finalize and start
    .await
    .unwrap();

app.shutdown().await.unwrap();
```

| Method | Purpose |
|--------|---------|
| `.module::<T>()` | Register a module (call once per module) |
| `.override_provider::<T>(val)` | Replace a provider of type `T` with `val` |
| `.build()` | Finalize the container and start the application |
| `.shutdown()` | Tear down the container gracefully |

## TestResponse assertion API

Every `.send()` call returns a `TestResponse` with these methods:

```rust
let resp = app.get("/users/1").send().await;

// Assertions (panic on failure)
resp.assert_status(200);
resp.assert_json(&expected);
resp.assert_error("USER_NOT_FOUND");

// Extraction (returns values)
let status: u16 = resp.status();
let body: serde_json::Value = resp.json();
let headers: HeaderMap = resp.headers();
```

| Method | Returns | Behavior |
|--------|---------|----------|
| `.assert_status(code)` | `()` | Panics if status != code |
| `.assert_json(&T)` | `()` | Panics if body doesn't match |
| `.assert_error(code)` | `()` | Panics if error code doesn't match |
| `.status()` | `u16` | Returns the HTTP status code |
| `.json()` | `serde_json::Value` / `T` | Deserializes the response body |
| `.headers()` | `HeaderMap` | Returns all response headers |

## Fluent assertion API

```rust
// Status code
app.get("/health").send().await.assert_status(200);
app.get("/missing").send().await.assert_status(404);

// JSON body
app.get("/users/1").send().await.assert_json(&UserView {
    id: 1,
    name: "Alice".into(),
});

// Error code
app.get("/users/999").send().await.assert_error("USER_NOT_FOUND");

// Extract raw JSON
let body: serde_json::Value = app.get("/items").send().await.json();
assert_eq!(body.as_array().unwrap().len(), 3);
```

## Request builder methods

The test app provides fluent HTTP method builders:

```rust
// GET — simple path-based request
app.get("/users").send().await;

// POST — with JSON body
app.post("/users")
    .json(&CreateUserDto { name: "Bob".into() })
    .header("Authorization", "Bearer token-abc")
    .send()
    .await;

// PUT — update resource
app.put("/users/1")
    .json(&UpdateUserDto { name: Some("Updated".into()), ..Default::default() })
    .send()
    .await;

// DELETE — remove resource
app.delete("/users/1").send().await;
```

| Method | App Method | Builder Methods |
|--------|-----------|-----------------|
| GET | `app.get(uri)` | `.header(key, val)`, `.send()` |
| POST | `app.post(uri)` | `.json(payload)`, `.header(key, val)`, `.send()` |
| PUT | `app.put(uri)` | `.json(payload)`, `.header(key, val)`, `.send()` |
| DELETE | `app.delete(uri)` | `.header(key, val)`, `.send()` |

> `.header()` can be chained multiple times to set several headers before calling `.send()`.

## Mocking dependencies

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

## Testing POST/PUT/DELETE

```rust
// POST with JSON body
app.post("/users")
    .json(&CreateUserDto {
        name: "Bob".into(),
        email: "bob@example.com".into(),
    })
    .send()
    .await
    .assert_status(201);

// PUT with body
app.put("/users/1")
    .json(&UpdateUserDto {
        name: Some("Bob Updated".into()),
        ..Default::default()
    })
    .send()
    .await
    .assert_status(200);

// DELETE
app.delete("/users/1").send().await.assert_status(204);
```

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

## CI setup (GitHub Actions)

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - uses: Swatinem/rust-cache@v2

      - run: cargo test --all-features
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/test
          RUST_LOG: ironic=warn
```

> For unit tests that don't need external services, run `cargo test` without `--all-features` to skip integration test modules.

## Complete test example

```rust
#[cfg(test)]
mod tests {
    use ironic::TestApplication;

    async fn app() -> TestApplication {
        TestApplication::new::<crate::AppModule>()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn health_check_works() {
        let app = app().await;
        app.get("/health").send().await.assert_status(200);
        app.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn create_and_get_user() {
        let app = app().await;

        // Create
        let resp = app.post("/users")
            .json(&serde_json::json!({"name": "Alice", "email": "alice@test.com"}))
            .send()
            .await;
        assert_eq!(resp.status(), 201);
        let created: serde_json::Value = resp.json();

        // Get
        let id = created["id"].as_u64().unwrap();
        let get_resp = app.get(&format!("/users/{id}")).send().await;
        assert_eq!(get_resp.status(), 200);

        app.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn validation_errors_return_400() {
        let app = app().await;
        app.post("/users")
            .json(&serde_json::json!({"name": ""}))  // ← Empty name (invalid)
            .send()
            .await
            .assert_error("VALIDATION_FAILED");
        app.shutdown().await.unwrap();
    }
}
```

## Try it yourself

1. Write a test that creates a user and verifies it exists
2. Write a test that sends invalid data and checks for a 400 error
3. Mock a service to return hardcoded data
4. Verify all three tests pass with `ironic test`

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
- [x] Fluent assertions: `.assert_status()`, `.assert_json()`, `.assert_error()`
- [x] Mock dependencies by overriding providers
- [x] `TestModule` tests a single module in isolation
- [x] Tests run in microseconds — no network overhead
- [x] Builder API: `.module::<T>()`, `.override_provider::<T>(val)`, `.build()`, `.shutdown()`
- [x] Request builders: `.get()`, `.post()`, `.put()`, `.delete()`, `.json()`, `.header()`, `.send()`
- [x] Each test gets a fresh container — no state leakage between tests
- [x] GitHub Actions CI runs `cargo test --all-features` with service containers
