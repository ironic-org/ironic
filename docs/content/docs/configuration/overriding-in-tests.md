---
title: Overriding in Tests
description: Configure your application differently in tests — override files, env vars, and provider values.
---

# Overriding in Tests

Testing often requires different configuration than production. Ironic provides several ways to override configuration in tests.

## Inline JSON override

The simplest approach — use `.json()` to override specific values:

```rust
#[test]
fn test_with_custom_port() {
    let config: TestConfig = ConfigurationLoader::new()
        .file("config.toml")
        .json(r#"{"port": 9999}"#)
        .load()
        .unwrap();

    assert_eq!(config.port, 9999);
}
```

## Environment variables

Set env vars before loading config:

```rust
#[test]
fn test_with_env_override() {
    std::env::set_var("APP__PORT", "8888");
    std::env::set_var("APP__DATABASE__URL", "postgres://test/localhost");

    let config: TestConfig = ConfigurationLoader::new()
        .file("config.toml")
        .environment("APP")
        .load()
        .unwrap();

    assert_eq!(config.port, 8888);
    assert_eq!(config.database_url, "postgres://test/localhost");

    // Clean up
    std::env::remove_var("APP__PORT");
    std::env::remove_var("APP__DATABASE__URL");
}
```

## Separate test config file

Create a `config.test.toml` and use it in tests:

```rust
#[test]
fn test_with_test_profile() {
    // Requires config.test.toml in the project root
    let config: TestConfig = ConfigurationLoader::new()
        .file("config.toml")
        .profile("test")
        .load()
        .unwrap();
}
```

## Test module provider overrides

For integration tests, the `TestModule` lets you override DI providers:

```rust
use ironic::testing::*;

#[tokio::test]
async fn test_with_mock_service() {
    let app = TestApplication::builder()
        .module::<AppModule>()
        .override_provider(
            // Replace real database with mock
            ProviderDefinition::value(MockDatabase::new()),
        )
        .build()
        .await;

    let response = app.get("/users").send().await;
    assert_eq!(response.status(), 200);
}
```

## Best practices for test config

- **Don't commit test-specific env vars** — Use `.json()` overrides instead
- **Use separate test config files** — For complex test setups
- **Clean up env vars in tests** — Use a helper that restores original values
- **Use `TestModule` overrides** — For full integration tests with mocked services
- **Prefer `ConfigurationLoader` in unit tests** — Test the config loading itself
