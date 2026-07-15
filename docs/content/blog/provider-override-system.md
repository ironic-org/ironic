---
title: "Provider Override System — how test mocks and production config swaps work"
description: "A technical walkthrough of Ironic's provider override mechanism: how ProviderKey matching, three override strategies, and shared API across TestApplicationBuilder, FrameworkApplicationBuilder, and TestModuleBuilder enable seamless mocking."
date: "2026-07-15"
author: "Ironic Team"
---

# Provider Override System — how test mocks and production config swaps work

Dependency injection containers that lock you into a single provider implementation are an integration-test nightmare. Ironic's container supports overrides — a mechanism that lets you swap any registered provider during application construction, using the same API across the production builder, the in-process test builder, and the isolated module builder. The entire system is backed by `ProviderKey` type-id matching and a late-binding override pass that runs just before the container is frozen.

---

## Override matching: `ProviderKey` identity

Every provider in Ironic is identified by a `ProviderKey` — a type-erased wrapper around `TypeId` (`ironic-di/src/lib.rs:22-53`). When you call `ProviderDefinition::value::<UserService>(mock_service)`, the definition records `ProviderKey::of::<UserService>()`, capturing the concrete Rust type.

An override is only valid if its key matches a key already registered from the module graph. The `ContainerBuilder::override_with()` method (`ironic-di/src/lib.rs:373-383`) does a direct `HashMap` lookup by key. If the key is not found, it returns `RegistrationError::InvalidOverride` — the builder will not silently ignore a mismatched override. If the key exists, the existing definition is replaced in-place. This is the one rule: **override keys must match registered keys exactly**.

---

## Three override strategies

The override API surfaces three methods, each corresponding to a different provider construction pattern:

### `override_provider(ProviderDefinition)` — full definition

This is the most general form. You construct a complete `ProviderDefinition` — specifying scope, dependencies, and an erased factory — and the override replaces the original registration wholesale. It is used when you need full control over lifetime, dependency graph, or scope semantics. Both `FrameworkApplicationBuilder` (`application.rs:143`) and `TestApplicationBuilder` (`testing/application.rs:21`) accept this directly.

### `override_value::<T>(value)` — singleton concrete value

For the common case of swapping in a mock or stub, `override_value` wraps the caller-supplied value in a `ProviderDefinition::value()` (`ironic-di/src/lib.rs:175-188`). The value is placed behind an `Arc`, registered at `Scope::Singleton`, and has an empty dependency list. This is the API you use when your mock holds no dependencies and is ready to use immediately:

```rust
TestApplication::builder::<MyModule>()
    .override_value::<dyn UserRepository>(MockUserRepo::new())
    .build()
    .await?;
```

### `override_factory::<T>(scope, dependencies, factory)` — async factory

When your override needs to resolve other providers from the container — for example, a mock that wraps a real database connection pool — you use `override_factory`. It accepts a `Scope`, a `Vec<Dependency>`, and an async closure that receives a `Resolver` and returns `Result<T, ResolveError>` (`testing/application.rs:35-49`). The factory participates in the same resolution graph as any other provider, meaning it can inject dependencies registered by the module:

```rust
.override_factory::<UserService>(
    Scope::Singleton,
    vec![Dependency::required::<dyn UserRepository>()],
    |resolver| async move {
        let repo = resolver.resolve::<dyn UserRepository>().await?;
        Ok(UserService::new(repo))
    },
)
```

---

## How overrides flow into the container

All three builders — `FrameworkApplicationBuilder`, `TestApplicationBuilder`, and `TestModuleBuilder` — store overrides in a plain `Vec<ProviderDefinition>`. The pattern is identical across all three:

1. Each `override_*` method pushes a `ProviderDefinition` onto the `overrides` field.
2. At build time, the internal compilation function (`build_http_application_with_overrides` or `build_http_application_with_extra_providers`, `lib.rs:587-610` and `617-645`) registers all module providers and controller providers into a `ContainerBuilder`, iterates the override list, and calls `container.override_with(provider)` for each one.
3. The `ContainerBuilder::build()` call consumes the final `HashMap<ProviderKey, ProviderDefinition>` and freezes it into an immutable `Container`.

The order matters: overrides are applied **after** all module registrations and extra provider injections. This guarantees that an override's `ProviderKey` matches an existing entry (otherwise it errors), and that the override definition is the one that survives into the frozen container.

---

## Shared API across three builders

The elegance of the system is that all three builders carry the same override surface. `TestApplicationBuilder` (`testing/application.rs:13-16`), `FrameworkApplicationBuilder` (`application.rs:75-79`), and `TestModuleBuilder` (`testing/module.rs:26-29`) each own `overrides: Vec<ProviderDefinition>`. The `TestApplicationBuilder::build()` method (`testing/application.rs:57-67`) delegates directly to `FrameworkApplication::builder()`, passing its collected overrides through the chain. The `TestModuleBuilder::compile()` method (`testing/module.rs:69-86`) calls `build_http_application_with_overrides`, the same function used by the production build path.

There is no separate "test DI container" or "mock registration API." The override mechanism is the same mechanism, whether you are building for production or for a unit test.

---

## Concrete example: swapping UserService for a mock

Assume a module that registers `UserService` with a constructor that depends on `dyn UserRepository`:

```rust
// module definition — production
module.register(ProviderDefinition::constructor::<UserService, _, _>(
    Scope::Singleton,
    vec![Dependency::required::<dyn UserRepository>()],
    |resolver| {
        let repo = resolver.resolve::<dyn UserRepository>()?;
        Ok(UserService::new(Arc::new(repo)))
    },
));
```

In test code, you swap the entire `UserService` for a mock that returns canned data:

```rust
#[tokio::test]
async fn user_endpoint_returns_mocked_data() {
    let mock_service = MockUserService::default();
    // Pre-configure the mock to return a known user
    mock_service.expect_get_user()
        .returning(|_| Ok(User { id: 1, name: "Test".into() }));

    let app = TestApplication::builder::<UserModule>()
        .override_value::<UserService>(mock_service)
        .build()
        .await
        .unwrap();

    let response = app.get("/users/1").await;
    assert_eq!(response.status(), 200);
    assert!(response.body().contains("Test"));
}
```

Because `override_value` wraps the mock in a `ProviderDefinition::value()`, the container treats it as a pre-resolved singleton. The `UserService` that the controller resolves at request time is the mock — no conditional compilation, no `#[cfg(test)]` gates, and no wiring changes between test and production code.

---

## Summary

Ironic's provider override system gives you exactly one rule: the override key must match a registered provider key. Beyond that, you can replace any provider with a static value, an async factory, or a full `ProviderDefinition`. The same `Vec<ProviderDefinition>` flows through the same `ContainerBuilder::override_with()` path whether you use `TestApplicationBuilder`, `FrameworkApplicationBuilder`, or `TestModuleBuilder`. This means test mocks are not a separate concept — they are simply overrides applied through the same API that powers configuration-driven provider swaps in production.
