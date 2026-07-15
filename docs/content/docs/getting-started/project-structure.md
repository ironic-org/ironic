---
title: Project Structure & Organization
description: A complete guide to every file and folder in an Ironic project — what they do, how they connect, and best practices for scaling your codebase.
---

# Project Structure & Organization

## What you'll learn

- What every top-level file and folder does
- How modules, controllers, services, repositories, DTOs, and entities fit together
- How data flows through a module (request → response)
- Best practices for organizing code as your project grows
- When to split a module into sub-modules

## The big picture

An Ironic project follows a **layered architecture**. Think of it like an assembly line:

```
 HTTP Request
     │
     ▼
  ┌────────────┐
  │ Controller │  ← Routes requests, validates input
  └─────┬──────┘
        │
  ┌─────▼──────┐
  │  Service   │  ← Business logic, orchestrates operations
  └─────┬──────┘
        │
  ┌─────▼──────────┐
  │  Repository    │  ← Data access (in-memory, database, API)
  └─────┬──────────┘
        │
  ┌─────▼──────┐
  │  Store     │  ← Database / external storage
  └────────────┘
```

Each layer only talks to the layer directly below it. This keeps your code testable, maintainable, and easy to refactor.

---

## Project overview

After running `ironic new my-app`, you get:

```
my-app/
├── Cargo.toml              # Rust dependencies & feature flags
├── ironic.toml             # Ironic project configuration
├── rust-toolchain.toml     # Rust version pinning
├── .env.example            # Environment variable template
├── .gitignore
├── Dockerfile              # Production container image
├── docker-compose.yml      # Local dev services (postgres, redis)
├── Makefile                # Quick command aliases
├── justfile                # Alternative to Makefile (just)
├── README.md               # Project documentation
├── .github/
│   └── workflows/
│       └── ci.yml          # GitHub Actions CI pipeline
└── src/
    ├── main.rs             # Application entry point
    ├── app.rs              # Root module — wires everything together
    ├── welcome.rs          # Default health check endpoint
    ├── platform/           # Cross-cutting infrastructure
    │   ├── mod.rs
    │   ├── config.rs       # Environment variable helpers
    │   ├── telemetry.rs    # Logging & tracing setup
    │   └── database.rs     # Database pool (when enabled)
    └── modules/            # Feature modules
        ├── mod.rs          # Module registry
        └── <module>/       # Each feature gets its own folder
            ├── mod.rs
            ├── controller/
            ├── services/
            ├── repositories/
            ├── dto/
            ├── entities/
            └── tests/
```

Let's walk through each piece.

---

## Top-level files

### `Cargo.toml` — Dependencies & feature flags

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
rust-version = "1.97"

[dependencies]
ironic = { version = "0.4.1", features = ["security", "metrics", "validation"] }
serde = { version = "1", features = ["derive"] }
garde = "0.23"
dotenvy = "0.15"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

The `ironic` dependency is the only framework import. Enable optional features in the `features = [...]` array:
- `security` — CORS, rate limiting, security headers
- `compression` — Gzip/brotli response compression
- `metrics` — Prometheus metrics endpoint
- `validation` — Request body validation via `garde`
- `versioning` — URI/header/media-type API versioning
- `serialization` — Role-based field exposure
- `database` — SQLx/SeaORM/Diesel integration
- `auth` — Password hashing, JWT, OAuth2, sessions
- `realtime` — WebSocket gateways
- `resilience` — Retry, circuit breaker
- `cron` — Cron expression scheduling
- `scheduling` — Background task scheduling
- `cache` — In-memory caching
- `distributed` — Message queues, microservices, CQRS
- `telemetry` — OpenTelemetry tracing
- `openapi` — Auto-generate OpenAPI schemas + Swagger UI

### `ironic.toml` — Project configuration

```toml
[project]
name = "my-app"
source_root = "src"
default_module = "src/app.rs"

[generate]
module_path = "src/modules"
```

Tells the CLI where to find the root module and where to generate new resources. You rarely need to modify this.

### `rust-toolchain.toml` — Rust version pinning

```toml
[toolchain]
channel = "1.97"
components = ["rustfmt", "clippy"]
```

Pins the Rust version so every developer on your team uses the same compiler. This prevents "works on my machine" issues.

### `.env.example` — Environment template

```
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
RATE_LIMIT_MAX=100
CORS_ORIGINS=["http://localhost:5173"]
DATABASE_URL=postgres://user:pass@localhost:5432/my_app
```

Copy to `.env` and fill in real values. The `.env` file is gitignored by default.

### `Dockerfile` & `docker-compose.yml`

```dockerfile
# Dockerfile — multi-stage build for production
FROM rust:1.97-slim-bookworm AS builder
# ... builds the binary

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-app /app/
# ... runs the binary
```

`docker-compose.yml` runs the app alongside postgres and redis for local development. Use `make docker-up` to start everything.

### `Makefile` & `justfile`

Quick command aliases so you don't have to remember `cargo` flags:

| Command | What it does |
|---------|-------------|
| `make dev` | `cargo run` |
| `make test` | `cargo test` |
| `make build` | `cargo build --release` |
| `make fmt` | `cargo fmt` |
| `make clippy` | `cargo clippy -- -D warnings` |
| `make docker-build` | `docker build -t my-app .` |
| `make docker-up` | `docker compose up -d` |

---

## The `src/` directory

### `main.rs` — Application entry point

This is where your application starts. It does 3 things:

1. **Loads environment variables** — `dotenvy::dotenv().ok();`
2. **Initializes tracing** — `platform::telemetry::init_tracing();`
3. **Builds & starts the application** — creates an `AxumAdapter`, registers middleware (CORS, rate limiting, security headers), and calls `application.listen(addr).await`

```rust
#[ironic::main]
async fn main() {
    dotenvy::dotenv().ok();
    platform::telemetry::init_tracing();

    let application = FrameworkApplication::builder()
        .module(AppModule::definition())
        .middleware(SecurityHeadersMiddleware::new(...))
        .middleware(RateLimitMiddleware::new(...))
        .middleware(CorsMiddleware::new(...))
        .platform(AxumAdapter::new().compression().request_body_limit(5 * 1024 * 1024))
        .build().await.unwrap();

    application.listen("127.0.0.1:3000").await.unwrap();
}
```

The `#[ironic::main]` attribute sets up the async runtime and DI container. Your code runs inside this container — you don't manage threads or tokio yourself.

### `app.rs` — The root module

This is the **traffic controller** of your application. It imports all feature modules and tells Ironic what to wire together:

```rust
#[derive(Module)]
#[module(
    imports = [HealthModule, MetricsModule, WelcomeModule, ExampleModule,
               crate::modules::products::ProductsModule],
    providers = [],
    controllers = [],
    exports = [],
)]
pub struct AppModule;
```

Every time you generate a new resource (`ironic generate resource products`), the CLI automatically adds it to this module's `imports` array. If you delete a module, remove it from here — the compiler will tell you if you miss anything.

### `welcome.rs` — Health check endpoint

A simple controller that returns a welcome JSON payload at `GET /`. Every project gets this for free.

---

## The `platform/` layer

The `src/platform/` directory holds **cross-cutting infrastructure** — code that doesn't belong to any single feature module.

### `platform/config.rs` — Environment helpers

```rust
pub fn env(key: &str) -> Option<String>
pub fn env_parsed<T: FromStr>(key: &str, default: T) -> T
pub fn env_json_array(key: &str) -> Vec<String>
```

Convenience functions for reading environment variables. Used throughout the application to keep configuration centralized.

### `platform/telemetry.rs` — Logging setup

Initializes `tracing-subscriber` with sensible defaults (level filtering, structured output). Call `init_tracing()` once in `main.rs` — every log, span, and event goes through this pipeline.

### `platform/database.rs` — Database pool

When you enable the `database` feature, this file provides a singleton database pool. The `build_pool()` function reads `DATABASE_URL` from the environment, creates a connection pool, and runs any pending migrations.

---

## The `modules/` directory — Feature modules

This is where your application's business logic lives. Each feature gets its own folder with a consistent internal layout.

### Module anatomy

After running `ironic generate resource products`:

```
src/modules/products/
├── mod.rs                         # Module definition
├── controller/
│   ├── mod.rs                     # Re-exports the controller
│   └── products_controller.rs     # HTTP routes (GET, POST, PUT, DELETE)
├── services/
│   ├── mod.rs                     # Re-exports the service
│   └── products_service.rs        # Business logic
├── repositories/
│   ├── mod.rs                     # Re-exports the repository
│   └── products_repository.rs     # Data access
├── dto/
│   ├── mod.rs                     # Re-exports DTOs
│   ├── create_products_dto.rs     # Input validation for POST
│   └── update_products_dto.rs     # Input validation for PUT
├── entities/
│   ├── mod.rs                     # Re-exports entities
│   └── products.rs               # Data model
└── tests/
    ├── mod.rs                     # Test module declarations
    ├── unit.rs                    # Unit tests (no HTTP, fast)
    └── integration.rs             # Integration tests (full HTTP)
```

### `mod.rs` — The module's wiring

```rust
#[derive(Module)]
#[module(
    providers = [ProductsRepository, ProductsService],
    controllers = [ProductsController],
)]
pub struct ProductsModule;
```

This tells Ironic:
1. **`providers`** — classes that can be injected into other classes (repositories, services)
2. **`controllers`** — classes that handle HTTP routes

The `AppModule` then imports `ProductsModule`, and Ironic automatically wires them together. You never manually create instances — Ironic's DI container handles that.

### Data flow through a module

Here's what happens when a `POST /products` request arrives:

```
1. HTTP Request
        │
2. ProductsController::create()
   ─── Receives the request
   ─── Validates the body against CreateProductsDto (garde)
   ─── Calls ProductsService.create(dto)
        │
3. ProductsService::create()
   ─── Applies business rules (transform, validate, authorize)
   ─── Calls ProductsRepository.create(data)
        │
4. ProductsRepository::create()
   ─── Interacts with the data store (memory, SQL, API)
   ─── Returns the saved entity
        │
5. ProductsService returns the result
   ─── Maps entity to response format if needed
        │
6. ProductsController returns JSON
   ─── Ironic serializes the response
   ─── Sends HTTP response
```

```
┌───────────┐     ┌──────────┐     ┌──────────────┐     ┌──────────┐
│ Controller│────►│ Service  │────►│ Repository   │────►│  Store   │
└───────────┘     └──────────┘     └──────────────┘     └──────────┘
     │                 │                 │
  Validates         Business           Data
  input             rules              access
```

### `controller/` — HTTP route handlers

Each controller is a struct with route methods. The `#[controller("/prefix")]` attribute sets the URL prefix, and each method uses `#[get]`, `#[post]`, `#[put]`, `#[delete]`:

```rust
#[controller("/products")]
#[derive(Injectable)]
pub struct ProductsController {
    service: Arc<ProductsService>,  // ← Injected automatically
}

#[routes]
impl ProductsController {
    #[get]
    async fn list(&self) -> Result<Json<Vec<Products>>, HttpError> {
        Ok(Json(self.service.list()))
    }

    #[get("/:id")]
    async fn get(&self, #[param] id: u64) -> Result<Json<Products>, HttpError> {
        self.service.find(id).map(Json)
    }

    #[post]
    async fn create(&self, #[body] dto: CreateProductsDto) -> Result<Json<Products>, HttpError> {
        Ok(Json(self.service.create(dto)))
    }

    #[put("/:id")]
    async fn update(&self, #[param] id: u64, #[body] dto: UpdateProductsDto) -> Result<Json<Products>, HttpError> {
        self.service.update(id, dto).map(Json)
    }

    #[delete("/:id")]
    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> {
        self.service.delete(id)
    }
}
```

Key points:
- **`#[param]`** extracts path/query parameters
- **`#[body]`** extracts and validates the JSON body
- **`Arc<Service>`** is injected by DI — you never construct it yourself
- Return `Result<Json<T>, HttpError>` for proper error handling

### `services/` — Business logic

Services contain the **what** — the actual operations your API performs:

```rust
#[derive(Injectable)]
pub struct ProductsService {
    pub repository: Arc<ProductsRepository>,  // ← DI injects this
}

impl ProductsService {
    pub fn list(&self) -> Vec<Products> {
        self.repository.list()
    }

    pub fn create(&self, dto: CreateProductsDto) -> Products {
        self.repository.create(dto.name, dto.description)
    }
}
```

Services are where you put:
- Validation rules beyond simple type checking
- Authorization checks (`current_user.can_edit(product)?`)
- Cross-cutting logic (send email after creation, invalidate cache)
- Orchestration of multiple repositories

### `repositories/` — Data access

Repositories are the **where** — they talk to storage. The generated starter uses an in-memory `Mutex<HashMap>`:

```rust
pub struct ProductsRepository;

impl ProductsRepository {
    pub fn list(&self) -> Vec<Products> { /* ... */ }
    pub fn find(&self, id: u64) -> Result<Products, HttpError> { /* ... */ }
    pub fn create(&self, name: String, desc: Option<String>) -> Products { /* ... */ }
    pub fn update(&self, id: u64, name: Option<String>, desc: Option<String>) -> Result<Products, HttpError> { /* ... */ }
    pub fn delete(&self, id: u64) -> Result<(), HttpError> { /* ... */ }
}
```

In production, you'd swap this for a database-backed repository. Because the controller talks to the service and the service talks to the repository, you can change the storage layer without touching routes or business logic.

### `dto/` — Data Transfer Objects

DTOs define what data the API accepts. They use `garde` for validation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProductsDto {
    #[garde(length(min = 1, max = 256))]
    pub name: String,

    #[garde(skip)]
    pub description: Option<String>,
}
```

- **`Create*Dto`** — fields required to create a resource (name required)
- **`Update*Dto`** — fields for partial updates (all fields optional)

### `entities/` — Data models

Entities represent the shape of your data, not what the API exposes:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Products {
    pub id: u64,
    pub name: String,
    pub description: String,
}
```

The key difference:
- **DTOs** = what the user sends (validation rules live here)
- **Entities** = what the database stores (structural definition)

### `tests/` — Automated tests

Every generated module comes with two test files:

**Unit tests** (`tests/unit.rs`) — test the service in isolation:
```rust
#[test]
fn service_has_the_correct_name() {
    let svc = ProductsService;
    assert_eq!(svc.name(), "products");
}
```

**Integration tests** (`tests/integration.rs`) — test full HTTP round-trips:
```rust
#[tokio::test]
async fn list_endpoint_returns_empty_when_no_data() {
    let app = TestApplication::new::<ProductsModule>().await.unwrap();
    let response = app.get("/products").send().await;
    assert_eq!(response.status(), HttpStatus::OK);
    app.shutdown().await.unwrap();
}
```

Run all tests with `cargo test` or `make test`.

---

## How dependency injection wires everything together

Here's the complete chain for a `GET /products` request:

```
                 ┌──────────────────────────────────────────────────────────────┐
                 │  AppModule                                                   │
                 │                                                              │
                 │  imports = [ProductsModule, ...]                             │
                 │       │                                                      │
                 │       ▼                                                      │
                 │  ┌─────────────────┐   ┌──────────────┐   ┌───────────────┐  │
                 │  │ ProductsModule  │   │ ProductsRepo │   │ ProductsSvc   │  │
                 │  │                 │──►│ (Injectable) │──►│ (Injectable)  │  │
                 │  │ providers = [   │   └──────────────┘   └───────┬───────┘  │
                 │  │   ProductsRepo, │                            │           │
                 │  │   ProductsSvc   │               DI Container injects      │
                 │  │ ]              │               Arc<ProductsRepo>          │
                 │  │ controllers = [│                            │            │
                 │  │   ProductsCtrl │   ┌────────────────────────┘            │
                 │  │ ]             │   │                                     │
                 │  └────────────────┘   ▼                                     │
                 │  ┌──────────────────────────────────────────┐               │
                 │  │ ProductsController                        │              │
                 │  │   service: Arc<ProductsService> ← injected              │
                 │  │                                            │             │
                 │  │   #[get("/products")]                      │             │
                 │  │   async fn list(&self) → Json<Vec<...>>    │             │
                 │  └──────────────────────────────────────────┘               │
                 └──────────────────────────────────────────────────────────────┘
```

The DI container scans all imported modules, finds everything marked `#[derive(Injectable)]`, and wires them together. If a dependency is missing, you get a compile-time error.

---

## Scaling: When to split modules

As your project grows, a single module might become too large. Here's how to organize at different scales:

### Small project (1-5 resources)

```
src/modules/
├── mod.rs
├── products/
├── users/
└── orders/
```

A file per module is fine. Each module has controller, service, repository, DTOs, entities.

### Medium project (5-20 resources)

Group related resources into **domain folders**:

```
src/
├── main.rs
├── app.rs
├── platform/
│   ├── config.rs
│   └── telemetry.rs
├── modules/
│   ├── catalog/                ← Domain: product catalog
│   │   ├── mod.rs
│   │   └── products/
│   ├── sales/                  ← Domain: sales & orders
│   │   ├── orders/
│   │   ├── carts/
│   │   └── payments/
│   └── identity/               ← Domain: users & auth
│       ├── users/
│       ├── roles/
│       └── sessions/
```

### Large project (20+ resources)

Split into **separate crates**:

```
my-project/
├── Cargo.toml                  ← Workspace root
├── crates/
│   ├── platform/               ← Shared infrastructure
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── config.rs
│   │       └── telemetry.rs
│   ├── catalog/                ← Domain crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── modules/
│   ├── sales/                  ← Domain crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── modules/
│   └── api/                    ← Application crate
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── app.rs          ← Imports domain crates
```

---

## File naming conventions

| Pattern | Example | When to use |
|---------|---------|-------------|
| `kebab-case` for module names | `src/modules/products/` | Directory names |
| `snake_case` for file names | `products_controller.rs` | All `.rs` files |
| `PascalCase` for types | `ProductsController` | Structs, enums, traits |
| `snake_case` for functions | `list()`, `create()` | Methods, functions |
| `PascalCase` with `Dto` suffix | `CreateProductsDto` | DTO structs |

---

## Best practices

### Keep controllers thin

Controllers should only:
1. Extract parameters (`#[param]`, `#[body]`)
2. Call a service method
3. Return the result as JSON

Move all business logic to services.

### Keep repositories behind traits

```rust
#[async_trait]
pub trait ProductRepository {
    async fn list(&self) -> Vec<Product>;
    async fn find(&self, id: u64) -> Result<Product, HttpError>;
}
```

This makes it easy to swap implementations (in-memory ↔ PostgreSQL ↔ mock).

### Name files after their main type

`products_controller.rs` contains `ProductsController`. If you open a file, you know exactly what's inside.

### Delete unused boilerplate

When you delete a module:
1. Remove its folder from `src/modules/`
2. Remove the `mod <name>;` line from `src/modules/mod.rs`
3. Remove it from the `imports` array in `src/app.rs`
4. Remove unused DTOs, entities, services

The compiler will catch anything you miss.

---

## Try it yourself

1. Run `ironic new bookstore` and explore every file
2. Generate 3 resources: `books`, `authors`, `reviews`
3. Open `src/app.rs` — see how the CLI auto-registered each module
4. Trace the data flow: open the controller → service → repository for `books`
5. Run `ironic test` and watch all auto-generated tests pass

---

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Putting database logic in a controller | Move it to a service, then a repository |
| Manually constructing services with `new()` | Use DI: add `Arc<YourService>` to the constructor |
| Forgetting to register a module in `app.rs` | Add it to `imports = [...]` in `#[module]` |
| Deleting a module folder but not removing from `app.rs` | Remove from `imports` — compiler catches missing files |
| Using `unwrap()` in controller methods | Return `Result<Json<T>, HttpError>` instead |

---

## What you learned

- [ ] The purpose of every top-level file: `Cargo.toml`, `ironic.toml`, `Dockerfile`, etc.
- [ ] The 3 layers: platform (infrastructure), modules (features), wiring (app.rs)
- [ ] The 6 parts of a module: controller, service, repository, DTO, entity, tests
- [ ] How data flows through a module: request → controller → service → repository → store
- [ ] How DI connects everything: AppModule → FeatureModule → Injectable classes
- [ ] How to scale from 1 module to a workspace of domain crates

## Next steps

Now that you understand the project structure, learn how each building block works:

→ [Core Concepts: Modules, Controllers, Services & DI](./fundamentals)
