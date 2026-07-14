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

## TestApp vs TestModule vs TestApplication

| Type | When to use |
|------|------------|
| `TestApplication::new::<AppModule>()` | Full integration test with all modules |
| `TestApplication::builder().override_provider()` | Replace specific services with mocks |
| `TestModule::new::<Module>()` | Test one module in isolation |

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

## What you learned

- [x] `TestApplication` runs full integration tests without a real server
- [x] Fluent assertions: `.assert_status()`, `.assert_json()`, `.assert_error()`
- [x] Mock dependencies by overriding providers
- [x] `TestModule` tests a single module in isolation
- [x] Tests run in microseconds — no network overhead
