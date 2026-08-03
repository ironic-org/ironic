---
title: Single Service
description: File-by-file breakdown of a single-service Ironic project
---

# Single Service Project

Generated with `ironic new my-app`:

```
my-app/
├── Cargo.toml              # ── Package manifest
├── ironic.toml             # ── Framework configuration
├── .env                    # ── Local env vars (git-ignored)
├── .env.example            # ── Documented env template
├── .gitignore
├── src/
│   ├── main.rs             # ── Entry point
│   ├── lib.rs              # ── Library root
│   ├── app.rs              # ── Root module
│   └── modules/            # ── Feature modules
│       ├── mod.rs          #    Module registry
│       └── <domain>/
│           ├── mod.rs
│           ├── controller/
│           ├── services/
│           ├── repositories/
│           ├── dto/
│           └── entities/
└── tests/
    ├── mod.rs
    ├── unit.rs
    └── integration.rs
```

## File-by-File Breakdown

### `Cargo.toml` — Package Manifest

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"

[dependencies]
ironic = { version = "1.1", features = ["security", "openapi"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
tracing-subscriber = "0.3"
```

Key sections:
- `[package]` — Standard Cargo metadata
- `[dependencies]` — Framework + ecosystem crates
- Feature flags enable specific Ironic capabilities

### `src/main.rs` — Application Entry Point

```rust
use ironic::prelude::*;

#[ironic::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Build and serve the application
    Application::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build()
        .await?
        .listen("0.0.0.0:3000")
        .await?;

    Ok(())
}
```

What happens at startup:
1. `#[ironic::main]` sets up the Tokio runtime
2. `Application::builder()` creates the application builder
3. `.module(AppModule::definition())` sets the root module
4. `.platform(AxumAdapter::new())` selects the HTTP platform (Axum)
5. `.build().await` compiles the module graph and initializes all providers
6. `.listen("0.0.0.0:3000")` starts the HTTP server

### `src/lib.rs` — Library Root

```rust
pub mod app;
pub mod modules;

// Re-export for integration tests
pub use app::AppModule;
```

Required for integration tests to access application internals.

### `src/app.rs` — Root Module

```rust
use ironic::prelude::*;

#[derive(Module)]
#[module(
    imports = [modules::users::UsersModule, modules::health::HealthModule],
)]
pub struct AppModule;
```

The root module:
- Imports all feature modules
- Declares global providers
- Exports providers to child modules

### `src/modules/mod.rs` — Module Registry

```rust
pub mod users;
pub mod health;
```

Every feature module is declared here and imported by `app.rs`.

### `src/modules/<domain>/` — Feature Module

A feature module contains the full vertical slice for one domain:

```
users/
├── mod.rs                 # Module definition
├── controller/
│   ├── mod.rs
│   └── users_controller.rs    # HTTP routes
├── services/
│   ├── mod.rs
│   └── users_service.rs       # Business logic
├── repositories/
│   ├── mod.rs
│   └── users_repository.rs    # Data access
├── dto/
│   ├── mod.rs
│   ├── create_user_dto.rs     # Request validation
│   └── user_response_dto.rs   # Response serialization
└── entities/
    ├── mod.rs
    └── user.rs                # Domain model
```

### `tests/` — Test Directory

```
tests/
├── mod.rs               # Test module declarations
├── unit.rs              # Unit tests (no HTTP)
└── integration.rs       # Full HTTP request/response tests
```

Example integration test:

```rust
use ironic::testing::*;

#[tokio::test]
async fn test_create_user() {
    let app = TestApplication::builder()
        .module(AppModule::definition())
        .build()
        .await;

    let response = app
        .post("/users")
        .json(&serde_json::json!({"name": "Alice"}))
        .await;

    assert_eq!(response.status(), 201);
}
```

## Environment Files

### `.env`

```bash
# Database
DATABASE_URL=postgres://localhost:5432/my_app

# Redis
REDIS_URL=redis://localhost:6379

# Auth
JWT_SECRET=development-secret-change-in-production

# Server
HOST=0.0.0.0
PORT=3000
LOG_LEVEL=info
```

### `ironic.toml`

```toml
[application]
name = "my-app"
version = "0.1.0"

[logging]
format = "json"
level = "info"

[openapi]
title = "My App API"
version = "0.1.0"
```
