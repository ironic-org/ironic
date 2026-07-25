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
│   │   └── src/
│   │       └── main.rs
│   ├── auth-service/              #    Authentication service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   └── analytics-service/         #    Analytics service
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
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
ironic = { git = "https://github.com/ironic-org/ironic", tag = "v1.1.1" }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net"] }
serde = { version = "1", features = ["derive"] }
tonic = "0.14"
prost = "0.13"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
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
proto = { path = "../../libs/proto" }
shared-config = { path = "../../libs/shared-config" }
tonic = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
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
# Build all services
cargo build --workspace

# Test all services
cargo test --workspace

# Build a single service
cargo build -p auth-service

# Run a single service
cargo run -p api-gateway
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
