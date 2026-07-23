---
title: Fundamentals Overview
description: The 4 building blocks of Ironic — Modules, Controllers, Services, and Dependency Injection.
---

# Fundamentals Overview

Every Ironic application is built from four core concepts. Understanding these is the key to mastering the framework.

## The 4 building blocks

| Building Block | What it does | Real-world analogy |
|---------------|-------------|-------------------|
| **Module** | Groups related code together | A department in a company |
| **Controller** | Handles HTTP requests (GET, POST, etc.) | The reception desk |
| **Service** | Contains business logic | The workers in the back office |
| **DI (Dependency Injection)** | Connects services to controllers automatically | The company org chart |

## How they fit together

```
┌─────────────────────────────────────────┐
│              AppModule                  │  ← Top-level: imports everything
│  ┌───────────────────────────────────┐  │
│  │         UsersModule               │  │  ← Feature: groups related code
│  │  ┌──────────┐  ┌───────────────┐  │  │
│  │  │Controller│  │   Service     │  │  │
│  │  │ (routes) │◄─│(business logic)│  │  │  ← Inside: the actual code
│  │  └──────────┘  └───────────────┘  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## Application anatomy

```
src/
├── main.rs              # Entry point: creates Application
├── app.rs               # Root module definition
└── users/
    ├── mod.rs           # Module definition with DI wiring
    ├── controller.rs    # HTTP routes
    └── service.rs       # Business logic with #[injectable]
```

## Module wiring

Modules declare what controllers and services they provide, and what other modules they need:

```rust
impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .import::<DatabaseModule>()          // depends on DatabaseModule
            .provider(ProviderDefinition::value(  // registers UserService
                UserService::new,
            ))
            .controller(UsersController::definition())
            .build()
    }
}
```

## Dependency Injection

Providers are resolved automatically by type. The container handles singleton, transient, and request-scoped lifetimes:

```rust
#[injectable]
impl UserService {
    fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
}
```

## Next steps

- [Modules](/docs/fundamentals/modules) — Deep dive into the module system
- [Providers](/docs/fundamentals/providers) — Provider scopes, registration, health
- [Lifecycle](/docs/fundamentals/lifecycle) — Application startup, runtime, shutdown
- [Request Lifecycle](/docs/fundamentals/request-lifecycle) — How a request flows through the pipeline
