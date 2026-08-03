---
title: CI Setup
description: Run Ironic tests in GitHub Actions — service containers, all-features builds, and a complete test example.
---

# CI Setup

## What you'll learn

- Configure GitHub Actions to run Ironic tests
- Spin up database service containers for integration tests
- Write a complete in-process test suite

---

## GitHub Actions

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

## Test hygiene

- Wrap every test module in `#[cfg(test)] mod tests { ... }` so it doesn't compile in release builds
- Prefer `TestApplication` for end-to-end flows, `TestModule` for single-controller tests
- Always call `.shutdown().await` to release connections and stop background tasks
- Keep tests independent — each test starts a fresh container

## What you learned

- [x] GitHub Actions CI runs `cargo test --all-features` with service containers
- [x] In-process tests need no ports, sockets, or Docker for the app itself
- [x] A reusable `app()` helper reduces boilerplate across tests
