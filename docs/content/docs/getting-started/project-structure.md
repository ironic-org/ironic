---
title: Project Structure
description: How Ironic projects are organized — from single services to monorepo workspaces
---

# Project Structure

Ironic supports two project layouts depending on your scale:

- **Single Service** — A standalone binary crate for a single application
- **Monorepo Workspace** — Multiple services sharing code via a workspace

## Single Service Structure

A single-service project created with `ironic new my-app`:

```
my-app/
├── Cargo.toml           # Package manifest
├── ironic.toml          # Ironic framework configuration
├── .env                 # Environment variables (git-ignored)
├── .env.example         # Documented environment template
├── src/
│   ├── main.rs          # Entry point — starts the application
│   ├── app.rs           # Root module definition
│   ├── lib.rs           # Library root (re-exports)
│   └── modules/         # Feature modules
│       ├── mod.rs       # Module registry
│       ├── users/       # Example module
│       │   ├── mod.rs
│       │   ├── controller/
│       │   ├── services/
│       │   ├── repositories/
│       │   ├── dto/
│       │   └── entities/
│       └── products/
│           └── ...
└── tests/
    ├── mod.rs
    └── integration.rs
```

### Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Application entry point. Uses `#[ironic::main]` to bootstrap the runtime |
| `src/app.rs` | Root module that imports all feature modules |
| `src/lib.rs` | Library root — re-exports for testing |
| `src/modules/` | One subdirectory per domain module (users, products, etc.) |
| `ironic.toml` | Framework-level configuration (log levels, feature toggles, etc.) |
| `.env` | Local environment overrides (never committed) |

## Monorepo Workspace Structure

For multiple microservices that share code, use the monorepo layout.
Create it with `ironic new my-platform` and add services with `ironic generate app <name>`:

```
my-platform/
├── Cargo.toml              # Workspace manifest (lists all members)
├── Cargo.lock              # Single lock file for the entire workspace
├── Makefile                # Unified build/test/deploy commands
├── docker-compose.yml      # Local development infrastructure
│
├── apps/                   # Microservice binaries
│   ├── api-gateway/        # HTTP API gateway (entry point)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── auth-service/       # Authentication microservice
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── analytics-service/  # Analytics microservice
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── libs/                   # Shared libraries
│   ├── shared-config/      # Configuration types shared across services
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── proto/              # Protobuf/gRPC type definitions
│       ├── Cargo.toml
│       ├── build.rs
│       ├── src/
│       │   └── lib.rs
│       └── proto/
│           └── greeter.proto
│
├── scripts/                # Shared CI/CD scripts
│   └── deploy.sh
│
└── docs/                   # Shared documentation
    └── architecture.md
```

### Workspace Cargo.toml

```toml
[workspace]
members = [
    "apps/api-gateway",
    "apps/auth-service",
    "apps/analytics-service",
    "libs/shared-config",
    "libs/proto",
]

[workspace.dependencies]
ironic = { git = "https://github.com/ironic-org/ironic", tag = "v1.1.1" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
serde = { version = "1", features = ["derive"] }
tonic = "0.14"
```

### Adding a New Microservice

```bash
# Creates apps/analytics-service/ with a scaffolded binary crate
ironic generate app analytics-service

# The command also registers it in the workspace Cargo.toml:
# [workspace]
# members = [
#     "apps/api-gateway",
#     "apps/analytics-service",   # ← added automatically
# ]
```

### Adding a Shared Library

```bash
# Creates libs/shared-config/ with a scaffolded library crate
ironic generate library shared-config

# Inside a monorepo, libraries go under libs/ automatically
# In a standalone project, they go next to src/
```

## Application Module Structure

Each microservice follows the same internal structure:

```
apps/auth-service/
├── Cargo.toml
└── src/
    ├── main.rs           # Entry point
    ├── app.rs            # Root module: imports + providers
    └── modules/
        ├── mod.rs        # Module registry
        ├── auth/         # Auth domain
        │   ├── mod.rs
        │   ├── controller/
        │   │   └── auth_controller.rs
        │   ├── services/
        │   │   └── auth_service.rs
        │   ├── repositories/
        │   │   └── user_repository.rs
        │   ├── dto/
        │   │   ├── mod.rs
        │   │   ├── login_dto.rs
        │   │   └── register_dto.rs
        │   └── entities/
        │       ├── mod.rs
        │       └── user.rs
        └── health/       # Health check module
            ├── mod.rs
            └── controller/
                └── health_controller.rs
```

### Entry Point (`src/main.rs`)

```rust
use ironic::prelude::*;

#[ironic::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

### Root Module (`src/app.rs`)

```rust
use ironic::prelude::*;

#[derive(Module)]
#[module(
    imports = [modules::auth::AuthModule, modules::health::HealthModule],
)]
pub struct AppModule;
```

## Microservice Communication Patterns

Services in a monorepo communicate through several mechanisms:

### 1. gRPC (Synchronous)

```
Service A ──gRPC call──▶ Service B
         ◀──response────
```

Define protobuf schemas in a shared library:

```rust
// libs/proto/src/lib.rs
tonic::include_proto!("greeter");
```

### 2. Event Bus (Asynchronous)

```
Service A ──event──▶ Kafka ──event──▶ Service B
```

Using the `#[event_handler]` macro:

```rust
#[event_handler(transport = "order.created")]
async fn handle_order_created(event: Arc<OrderEvent>) {
    // Process asynchronously
}
```

### 3. Redis Transport (Request-Reply)

```
Service A ──send──▶ Redis ──handle──▶ Service B
         ◀──reply───────────────────
```

```rust
// Service B
server.on_message("user.get", handler);
server.listen().await?;

// Service A
let user: User = client.send("user.get", &request).await?;
```

## Environment Configuration

Each service can have its own `.env` file:

```bash
# apps/api-gateway/.env
PORT=3000
DATABASE_URL=postgres://localhost:5432/platform

# apps/auth-service/.env
PORT=3001
JWT_SECRET=local-development-secret
REDIS_URL=redis://localhost:6379
```

The workspace root can have a `docker-compose.yml` for shared infrastructure:

```yaml
version: "3.9"
services:
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

  kafka:
    image: confluentinc/cp-kafka:latest
    ports: ["9092:9092"]
    environment:
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
```

## Best Practices

1. **One domain per module** — Each module in `src/modules/` owns a single domain concept
2. **Shared code in libraries** — Put shared types, protobuf definitions, and utilities in `libs/`
3. **Workspace dependencies** — Use `[workspace.dependencies]` to keep versions synchronized
4. **Feature flags per service** — Enable only the features each service needs
5. **CI across workspace** — Run `cargo test --workspace` to test all services together
6. **Independent deployability** — Each service has its own port, database, and lifecycle
