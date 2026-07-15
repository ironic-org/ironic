---
title: "Asynchronous Module Configuration — deferred root modules from remote sources"
description: "How Ironic's FrameworkApplicationBuilder::module_async() lets you load configuration from secrets managers, service discovery, or remote APIs before the module graph is compiled."
date: "2026-07-15"
author: "Ironic Team"
---

# Asynchronous Module Configuration — deferred root modules from remote sources

Most dependency injection containers want their configuration upfront and synchronous. You declare modules, register providers, and call `.build()` — all on the stack. But real applications rarely have all their configuration available at compile time. Database passwords live in AWS Secrets Manager. Service URLs come from Consul. Feature flags come from a remote API. You can't build a module graph until those values arrive.

Ironic solves this with a two-phase build: register a deferred root module, then await resolution before graph compilation.

---

## The problem: secrets aren't compile-time constants

Consider a typical production deployment. Your `DatabaseModule` needs a connection string. That string contains a username, password, and host — none of which you want in source control or environment files. You expose them through a secrets manager and fetch them at startup.

In most DI frameworks, this forces an awkward bootstrap dance: run a separate init step to fetch secrets, build a config struct, then pass it to the container. The framework has no knowledge of the async gap — it's entirely the application author's responsibility to bridge it.

Ironic bakes the gap into the builder API itself.

---

## The `RootModule` enum: Ready vs Deferred

At the heart of the mechanism is a two-variant enum in `crates/ironic-core/src/application.rs:85`:

```rust
type ModuleConfigurationFuture =
    Pin<Box<dyn Future<Output = Result<ModuleDefinition, ModuleConfigurationError>> + Send>>;

enum RootModule {
    Ready(ModuleDefinition),
    Deferred(ModuleConfigurationFuture),
}
```

When you call the standard `.module(definition)`, the root is set to `Ready(ModuleDefinition)` — same as any static DI setup. But when you call `.module_async(future)`, the root becomes `Deferred(ModuleConfigurationFuture)` — a boxed, pinned future that will eventually resolve to a `ModuleDefinition`. The framework stores this future without evaluating it.

---

## The public API: `module_async()`

The builder method at line 131 is straightforward:

```rust
pub fn module_async<F>(mut self, module: F) -> Self
where
    F: Future<Output = Result<ModuleDefinition, ModuleConfigurationError>> + Send + 'static,
{
    self.root = Some(RootModule::Deferred(Box::pin(module)));
    self
}
```

It accepts any future that returns `Result<ModuleDefinition, ModuleConfigurationError>`. The `ModuleConfigurationError` type wraps a sanitized error message — more on that later. The function boxes and pins the future, stores it in the builder, and returns. No network calls happen here. The builder is still a cheap struct on the stack.

---

## Awaiting at build time

The magic happens in `.build()` at line 182 — and notice that `.build()` is now `async`:

```rust
pub async fn build(self) -> Result<FrameworkApplication<A::Application>, ApplicationError> {
    let root = match self.root.ok_or(ApplicationError::MissingRootModule)? {
        RootModule::Ready(module) => module,
        RootModule::Deferred(module) => {
            module
                .await
                .map_err(|error| ApplicationError::ModuleConfiguration {
                    message: error.to_string(),
                })?
        }
    };
    // ... compile_module_graph(root), build providers, etc.
}
```

The `.await` on the deferred future is the point where the actual async work happens. Only after the future resolves does `compile_module_graph(root)` run. This means the full DI graph — provider validation, circular dependency detection, scope checks — all happen with the real runtime configuration in hand.

If the async future fails (e.g., the secrets manager is unreachable), the error is mapped to `ApplicationError::ModuleConfiguration` and the build fails cleanly.

---

## `ModuleConfigurationError`: credential-safe errors

A subtle but critical design detail: error messages from module configuration must never contain secrets. If a secret fetch fails with a response body containing a partially resolved API key, you don't want that logged. `ModuleConfigurationError` enforces this at the type level:

```rust
pub struct ModuleConfigurationError {
    message: String,
}

impl ModuleConfigurationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}
```

The constructor takes an explicit `message` string — not the original error. This forces the caller to sanitize. A typical usage pattern captures the outcome, logs the real error with care, and constructs a sanitized `ModuleConfigurationError` with a user-safe message like `"remote configuration is unavailable"`.

---

## Bridging static declarations with runtime configuration

The real power of this pattern is that it bridges two worlds without splitting the API. Your module definitions remain the same `ModuleDefinition` structs — same providers, same dependency declarations. The only thing that changes is *when* they're assembled.

Here's a concrete pattern for loading database credentials from a secrets manager:

```rust
use ironic_core::application::ModuleConfigurationError;

FrameworkApplication::builder()
    .module_async(async {
        let secret = aws_sdk_secretsmanager::Client::new(&config)
            .get_secret_value()
            .secret_id("prod/database")
            .send()
            .await
            .map_err(|_| ModuleConfigurationError::new(
                "failed to load database credentials",
            ))?;

        let conn_string = secret.secret_string().unwrap();

        Ok(ModuleDefinition::new("app")
            .register(ProviderDefinition::value(conn_string))
            .module(DatabaseModule::new()))
    })
    .platform(axum_adapter())
    .build()
    .await?;
```

The module graph isn't compiled until the secret is fetched. If the secrets manager is down, the application fails at startup with a clear error — not three hours later when the first database query times out with a bogus connection string.

---

## The trade-off

Deferred configuration adds startup latency. Every millisecond spent waiting for a secrets manager, service discovery query, or remote API call is a millisecond before your application is ready to serve traffic. For CLI tools that need to start instantly, this is a dealbreaker.

For long-running services, the trade-off is almost always worth it. The alternative — loading secrets synchronously before the container, constructing a config struct, and threading it through manually — doesn't eliminate the latency. It just moves the wait outside the framework, into boilerplate that every application author writes slightly differently. By pulling the async gap into the builder, Ironic standardizes the startup sequence and makes secret-manager integration a first-class feature rather than an afterthought.
