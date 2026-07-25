---
title: Monorepo Workspace
description: Complete scaffold breakdown for multi-service Ironic projects
---

# Monorepo Workspace

Generated with `ironic new my-platform` + `ironic generate app <name>`:

```
my-platform/
├── Cargo.toml                     # ── Workspace manifest
├── Cargo.lock                     # ── Single lock file
├── Makefile                       # ── Unified build commands
├── docker-compose.yml             # ── Shared infrastructure
├── .env                           # ── Root env vars
│
├── apps/                          # ── Microservice binaries
│   ├── api-gateway/               #    HTTP API gateway
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   ├── .env
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs             #    Root module
│   │       ├── welcome.rs         #    Welcome endpoint
│   │       ├── platform/          #    Service infrastructure
│   │       │   ├── mod.rs
│   │       │   ├── config.rs
│   │       │   ├── telemetry.rs
│   │       │   └── database.rs
│   │       └── modules/
│   │           ├── mod.rs
│   │           ├── users/         #    Users domain
│   │           │   ├── mod.rs
│   │           │   ├── controller/
│   │           │   │   ├── mod.rs
│   │           │   │   └── users_controller.rs
│   │           │   ├── services/
│   │           │   │   ├── mod.rs
│   │           │   │   └── users_service.rs
│   │           │   ├── repositories/
│   │           │   │   ├── mod.rs
│   │           │   │   └── users_repository.rs
│   │           │   ├── dto/
│   │           │   │   ├── mod.rs
│   │           │   │   ├── create_user_dto.rs
│   │           │   │   └── user_response.rs
│   │           │   ├── entities/
│   │           │   │   ├── mod.rs
│   │           │   │   └── user.rs
│   │           │   └── tests/     #    Unit + integration tests
│   │           │       ├── mod.rs
│   │           │       ├── unit.rs
│   │           │       └── integration.rs
│   │           └── health/        #    Health check
│   │               └── controller/
│   │                   └── health_controller.rs
│   │
│   ├── auth-service/              #    Authentication service
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   ├── .env
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs
│   │       ├── welcome.rs
│   │       ├── platform/
│   │       │   ├── mod.rs
│   │       │   ├── config.rs
│   │       │   ├── telemetry.rs
│   │       │   └── database.rs
│   │       └── modules/
│   │           ├── mod.rs
│   │           ├── auth/
│   │           │   ├── mod.rs
│   │           │   ├── controller/
│   │           │   │   ├── mod.rs
│   │           │   │   └── auth_controller.rs
│   │           │   ├── services/
│   │           │   │   ├── mod.rs
│   │           │   │   ├── auth_service.rs
│   │           │   │   └── jwt_service.rs
│   │           │   ├── repositories/
│   │           │   │   ├── mod.rs
│   │           │   │   └── credentials_repository.rs
│   │           │   ├── dto/
│   │           │   │   ├── mod.rs
│   │           │   │   ├── login_dto.rs
│   │           │   │   └── register_dto.rs
│   │           │   ├── entities/
│   │           │   │   ├── mod.rs
│   │           │   │   └── credential.rs
│   │           │   └── tests/
│   │           │       ├── mod.rs
│   │           │       ├── unit.rs
│   │           │       └── integration.rs
│   │           └── rbac/
│   │               ├── mod.rs
│   │               ├── services/
│   │               │   ├── mod.rs
│   │               │   └── rbac_service.rs
│   │               └── entities/
│   │                   ├── mod.rs
│   │                   └── role.rs
│   │
│   └── analytics-service/         #    Analytics service
│       ├── Cargo.toml
│       ├── Dockerfile
│       ├── .env
│       └── src/
│           ├── main.rs
│           ├── app.rs
│           ├── welcome.rs
│           ├── platform/
│           │   ├── mod.rs
│           │   ├── config.rs
│           │   ├── telemetry.rs
│           │   └── database.rs
│           └── modules/
│               ├── mod.rs
│               ├── events/        #    Kafka event handlers
│               │   ├── mod.rs
│               │   └── handlers/
│               │       ├── mod.rs
│               │       ├── user_events.rs
│               │       └── order_events.rs
│               └── reports/
│                   ├── mod.rs
│                   ├── controller/
│                   │   ├── mod.rs
│                   │   └── reports_controller.rs
│                   ├── services/
│                   │   ├── mod.rs
│                   │   └── reports_service.rs
│                   ├── repositories/
│                   │   ├── mod.rs
│                   │   └── analytics_repository.rs
│                   ├── dto/
│                   │   ├── mod.rs
│                   │   └── report_response.rs
│                   ├── entities/
│                   │   ├── mod.rs
│                   │   └── analytics_event.rs
│                   └── tests/
├── libs/                          # ── Shared libraries
│   ├── shared-config/             #    Shared config types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── proto/                     #    Protobuf definitions
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── src/lib.rs
│   │   └── proto/
│   │       └── greeter.proto
│   └── observability/             #    Shared tracing setup
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
│
├── scripts/                       # ── CI/CD scripts
│   └── deploy.sh
│
└── docs/                          # ── Shared documentation
    └── architecture.md
```

## Workspace Manifest (`Cargo.toml`)

```toml
[workspace]
resolver = "3"
members = [
    "apps/api-gateway",
    "apps/auth-service",
    "apps/analytics-service",
    "libs/shared-config",
    "libs/proto",
    "libs/observability",
]

[workspace.dependencies]
ironic = { version = "0.2", features = ["security", "compression", "metrics", "openapi"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net", "signal"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
garde = "0.23"
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres"] }
tracing = { version = "0.1", features = ["attributes"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
tonic = "0.14"
prost = "0.13"
```

Key points:
- `[workspace]` section defines all member crates
- `[workspace.dependencies]` keeps versions synchronized across all services
- Adding a new member: `ironic generate app <name>` or manually add to `members`

## Service Binary (`apps/<name>/Cargo.toml`)

```toml
[package]
name = "auth-service"
version = "0.1.0"
edition = "2024"

[dependencies]
ironic = { workspace = true, features = ["auth", "transport-redis"] }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
garde = { workspace = true }
sqlx = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
dotenvy = { workspace = true }
proto = { path = "../../libs/proto" }
shared-config = { path = "../../libs/shared-config" }
tonic = { workspace = true }
```

Each service only enables the Ironic features it needs.

## Shared Library (`libs/<name>/`)

### Library Manifest

```toml
[package]
name = "shared-config"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
ironic = { workspace = true }
```

### Library Source

```rust
// libs/shared-config/src/lib.rs
pub mod config;
pub mod types;
```

## How Controllers, Services, and Modules Behave in a Monorepo

In a monorepo, each service is its own `[[bin]]` crate with its own DI container,
module graph, and lifecycle. Unlike a single-service app where all modules share
one container, in a monorepo each service is fully isolated.

### Module Scope Per Service

```
┌─────────────────────────────────────────────┐
│               Monorepo Workspace             │
│                                              │
│  ┌─ Service A (api-gateway) ──────────────┐  │
│  │  Container A                            │  │
│  │  ┌──────────┐  ┌──────────┐            │  │
│  │  │ Users    │  │ Products │            │  │
│  │  │ Module   │  │ Module   │            │  │
│  │  └──────────┘  └──────────┘            │  │
│  └──────────────────────────────────────────┘  │
│                                              │
│  ┌─ Service B (auth-service) ─────────────┐  │
│  │  Container B                            │  │
│  │  ┌──────────┐  ┌──────────┐            │  │
│  │  │ Auth     │  │ Users    │  (different│  │
│  │  │ Module   │  │ Module   │   codebase)│  │
│  │  └──────────┘  └──────────┘            │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

**Key rule:** Each service compiles and runs independently. Modules in Service A
cannot directly inject providers from Service B. Cross-service communication
goes through the network (gRPC, Kafka, Redis, HTTP).

### Shared Libraries for Common Code

Code that should be shared across services lives in `libs/`:

```
libs/
├── proto/                    # gRPC type definitions
│   └── src/lib.rs            # tonic::include_proto!("greeter")
│
├── shared-config/            # Configuration structs
│   └── src/lib.rs
│       pub struct DatabaseConfig { ... }
│       pub struct RedisConfig { ... }
│
└── observability/            # Shared tracing/metrics setup
    └── src/lib.rs
        pub fn init_tracing() { ... }
```

### Controller in a Monorepo Service

Each service's controllers only handle routes for THAT service:

```rust
// apps/api-gateway/src/modules/users/controller/users_controller.rs
#[controller("/users")]
pub struct UsersController {
    // Service from the SAME crate's DI container
    users_service: Arc<UsersService>,
}

impl UsersController {
    // This controller calls a gRPC client to reach auth-service
    #[post("/")]
    async fn create(&self, #[body] dto: CreateUserDto) -> Json<UserResponse> {
        // 1. Call auth-service via gRPC for authentication
        let auth_response = self.auth_client
            .register(RegisterRequest { email: dto.email.clone() })
            .await?;

        // 2. Save user data locally
        let user = self.users_service.create(dto).await?;

        // 3. Emit event to Kafka for analytics-service
        self.event_bus.publish(UserCreated { id: user.id }).await;

        Json(UserResponse::from(user))
    }
}
```

### Service Layer in a Monorepo

Services use DI to inject other services, repositories, and clients from the SAME crate:

```rust
// apps/auth-service/src/modules/auth/services/auth_service.rs
#[derive(Injectable)]
pub struct AuthService {
    // Local repository (same crate)
    user_repo: Arc<UserRepository>,

    // Shared library types
    config: Arc<shared_config::config::AuthConfig>,

    // gRPC client to communicate with other services
    // (defined and injected within this same crate)
    analytics_client: Arc<AnalyticsClient>,
}

impl AuthService {
    pub async fn register(&self, email: String, password: String) -> Result<User, AuthError> {
        // 1. Local database operation
        let user = self.user_repo.create(email, password).await?;

        // 2. Cross-service communication via gRPC
        self.analytics_client
            .track_event(TrackEvent { event: "user.registered" })
            .await?;

        Ok(user)
    }
}
```

### Provider Registration Per Service

Each service registers its OWN providers. Shared libraries are pulled in as Cargo
dependencies but their types can be registered as providers:

```rust
// apps/auth-service/src/app.rs
#[derive(Module)]
#[module(
    providers = [
        AuthService,
        UserRepository,
        AnalyticsClient,
    ],
)]
pub struct AppModule;

// Shared library values can be registered in a separate config:
// Application::builder()
//     .provider(ProviderDefinition::value(
//         shared_config::config::AppConfig::from_env()
//     ))
```

### Cross-Service Call Flow Diagram

```
┌─ Service A (api-gateway) ──────────────────────┐
│                                                  │
│  HTTP Request ──▶ Controller                     │
│                      │                           │
│                      ▼                           │
│                   Service (local DI)              │
│                      │                           │
│         ┌────────────┼───────────────┐           │
│         ▼            ▼               ▼           │
│   Repository   gRPC Client    Event Bus          │
│   (local DB)   (to Service B)  (Kafka)           │
└──────────────────────────────────────────────────┘
                      │               │
                      │ gRPC          │ Kafka
                      ▼               ▼
┌─ Service B ─────────────┐   ┌─ Service C (analytics) ─┐
│  gRPC Server            │   │  Kafka Consumer          │
│  ▶ Service              │   │  ▶ Service               │
│    ▶ Repository         │   │    ▶ Repository          │
└─────────────────────────┘   └──────────────────────────┘
```

### Dependency Graph Summary

```
┌─────────────────────────────────────────────────────────┐
│                   WORKSPACE LEVEL                        │
│                                                         │
│  libs/proto ◀────── apps/service-a                      │
│      │                 │                                 │
│      │                 ├── depends on: ironic           │
│      │                 ├── depends on: libs/proto        │
│      │                 ├── depends on:                   │
│      │                 │   libs/shared-config            │
│      │                 │                                 │
│      ├────── apps/service-b                              │
│      │         │                                         │
│      │         ├── depends on: ironic                    │
│      │         ├── depends on: libs/proto                │
│      │         └── depends on: libs/shared-config        │
│      │                                                   │
│  libs/shared-config                                      │
│      │                                                   │
│      └────── all services                                │
└─────────────────────────────────────────────────────────┘
```

### Key Differences from Single-Service

| Aspect | Single Service | Monorepo |
|--------|---------------|----------|
| DI Container | One container for all modules | One container PER service |
| Module imports | Internal modules | Internal modules + shared libs |
| Cross-service calls | Direct function calls | gRPC / Kafka / Redis / HTTP |
| Shared code | Duplicated or extracted to lib/ | Shared via `libs/` workspace members |
| Testing | Single `cargo test` | `cargo test --workspace` |
| Deployment | One binary | Multiple binaries |

## Cross-Service Dependencies

```bash
# Service A depends on shared-config library
apps/service-a/
└── Cargo.toml
    [dependencies]
    shared-config = { path = "../../libs/shared-config" }
```

## Build & Test Commands

```bash
# Build the current project
ironic build

# Build all services in workspace
ironic build -- --workspace

# Run the current project
ironic start

# Run a specific app in the workspace
ironic start -p api-gateway           # cargo run -p api-gateway

# Run with release profile
ironic start -- --release

# Run a specific app with features
ironic start -p auth-service -- --features transport-redis

# Test all services
cargo test --workspace
```

## Makefile

```makefile
.PHONY: build test run-all

build:
	cargo build --workspace

test:
	cargo test --workspace

run-all:
	@echo "Starting all services..."
	cargo run -p api-gateway &
	cargo run -p auth-service &
	cargo run -p analytics-service &
	wait

run-%:
	cargo run -p $*

docker-up:
	docker compose up -d

docker-down:
	docker compose down
```

## Docker Compose

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

  postgres:
    image: postgres:16-alpine
    ports: ["5432:5432"]
    environment:
      POSTGRES_USER: platform
      POSTGRES_PASSWORD: development
      POSTGRES_DB: platform
