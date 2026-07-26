---
title: Feature Reference
description: Complete API reference for all Ironic features, modules, re-exports, and how to use them in your application.
---

# Feature Reference

Ironic is a batteries-included framework. Everything your application needs comes through the `ironic` crate. This document explains every feature flag, every public module, every re-exported crate, and how to use them.

---

## Table of Contents

1. [Feature Flags](#feature-flags)
2. [Public Modules](#public-modules)
3. [Re-exported Crates](#re-exported-crates)
4. [Core Types & Traits](#core-types--traits)
5. [Derive Macros](#derive-macros)
6. [Attribute Macros](#attribute-macros)
7. [Helper Macros](#helper-macros)
8. [Application Lifecycle](#application-lifecycle)
9. [Dependency Injection](#dependency-injection)
10. [Module System](#module-system)
11. [Routing](#routing)
12. [Middleware Stack](#middleware-stack)
13. [Security](#security)
14. [Observability](#observability)
15. [Database & Storage](#database--storage)
16. [Transports](#transports)

---

## Feature Flags

```toml
[dependencies]
ironic = { version = "1.1", features = ["..."] }
```

### Default Features

| Feature | Description | Dependencies Enabled |
|---------|-------------|---------------------|
| `hot-reload` | File watching for `ironic dev` | `notify` |
| `openapi` | OpenAPI 3.1 spec generation + Swagger UI | `ironic-openapi` |
| `logging` | Structured logging with `tracing` | `tracing-subscriber` |

### Core Features

| Feature | Description | Dependencies Enabled |
|---------|-------------|---------------------|
| `validation` | Request body validation with `garde` | `garde` |
| `serialization` | Role-based field exposure | (no external deps) |
| `security` | CORS, rate limiting, security headers | `ironic-security` |
| `metrics` | Prometheus metrics endpoint | `ironic-metrics` |
| `scheduling` | Fixed-interval background tasks | (no external deps) |
| `cron` | Cron expression scheduling | `cron` |

### Database Features

| Feature | Description | Dependencies Enabled |
|---------|-------------|---------------------|
| `sqlx-postgres` | PostgreSQL via SQLx | `sqlx`, `sqlx/postgres` |
| `sqlx-mysql` | MySQL via SQLx | `sqlx`, `sqlx/mysql` |
| `sqlx-sqlite` | SQLite via SQLx | `sqlx`, `sqlx/sqlite` |
| `seaorm-postgres` | PostgreSQL via SeaORM | `sea-orm`, `sea-orm/sqlx-postgres` |
| `seaorm-mysql` | MySQL via SeaORM | `sea-orm`, `sea-orm/sqlx-mysql` |
| `diesel` | Diesel ORM | `diesel` |
| `mongodb` | MongoDB driver | `mongodb` |
| `redis` | Redis driver | `redis` |

### Transport Features

| Feature | Description | Dependencies Enabled |
|---------|-------------|---------------------|
| `grpc` | gRPC services with tonic | `tonic`, `prost` |
| `graphql` | GraphQL APIs with async-graphql | `async-graphql`, `async-graphql-derive` |
| `sse` | Server-Sent Events | (internal module) |
| `realtime` | WebSocket gateways | `axum/ws` |
| `mqtt` | MQTT transport | `rumqttc` |
| `nats` | NATS transport | `async-nats` |
| `kafka` | Kafka transport | `kafka` |
| `rabbitmq` | RabbitMQ transport | `lapin` |

### Distributed Systems Features

| Feature | Description | Dependencies Enabled |
|---------|-------------|---------------------|
| `queues` | Redis-backed job queues | `redis` |
| `events` | Event bus (in-process) | (internal module) |
| `cqrs` | CQRS pattern support | (internal module) |
| `sagas` | Saga orchestration | (internal module) |
| `microservices` | Microservice scaffolding | (internal module) |
| `distributed` | All distributed features combined | `queues`, `microservices`, `cqrs`, `sagas`, `grpc`, `graphql` |

### Auth Features

| Feature | Description | Dependencies Enabled |
|---------|-------------|---------------------|
| `auth` | Full auth: passwords, JWT, OAuth, sessions, RBAC | `argon2`, `jsonwebtoken`, `oauth2` |
| `jwt` | JWT token management | `jsonwebtoken` |
| `sessions` | Session management | `getrandom` |

### Other Features

| Feature | Description | Dependencies Enabled |
|---------|-------------|---------------------|
| `cache` | In-memory + Redis cache interceptor | (internal module) |
| `multipart` | File upload handling | `multer` |
| `static-files` | Static file serving | `tower-http/fs` |
| `serverless` | AWS Lambda support | `lambda_http` |
| `mcp` | MCP protocol server | (internal module) |
| `telemetry` | OpenTelemetry distributed tracing | `opentelemetry`, `opentelemetry_sdk` |
| `resilience` | Retry + circuit breaker | `ironic-resilience` |
| `versioning` | API versioning | (internal module) |
| `devtools` | Development tooling | (internal module) |
| `plugins` | Plugin ecosystem | (internal module) |
| `uuid` | UUID generation | `uuid` |

---

## Public Modules

All public modules are accessed via `ironic::module_name`.

### Core Modules

#### `ironic::prelude`

The prelude is the primary import. It brings in the most commonly used types:

```rust
use ironic::prelude::*;
// Now available without further imports:
//   Application, Module, Injectable, ControllerDefinition
//   HttpError, Json, RequestContext, Response
//   all derive macros (Module, Injectable)
//   all attribute macros (get, post, controller, routes)
```

**What the prelude exports:**

| Type | Kind | Description |
|------|------|-------------|
| `Application` | struct | Main application builder |
| `AxumAdapter` | struct | HTTP platform adapter for Axum |
| `Module` | derive macro | Define a module |
| `Injectable` | derive macro | Make a struct injectable |
| `ControllerDefinition` | struct | Route controller metadata |
| `HttpError` | struct | HTTP error with code + message |
| `Json` | struct | JSON response wrapper |
| `RequestContext` | struct | Per-request context |
| `Response` | struct | HTTP response |
| `ModuleDefinition` | struct | Module configuration |
| `ModuleRef` | struct | Runtime module handle |
| `Middleware` | trait | Request middleware |
| `Guard` | trait | Route guard |
| `Interceptor` | trait | Request interceptor |
| `ExceptionFilter` | trait | Error handler |
| `ParameterPipe` | trait | Parameter transformation |
| `ParameterExtractor` | trait | Custom parameter extraction |
| `HealthModule` | struct | Built-in health check |
| `RequestLogging` | struct | Access logging middleware |
| `RequestId` | struct | Unique request ID |
| `Value` | type alias | `serde_json::Value` |
| `Scope` | enum | DI scope (Singleton, Transient, Request) |
| All proc macros | macros | `get`, `post`, `put`, `delete`, `controller`, `routes`, etc. |

#### `ironic::json`

Convenience re-exports from `serde_json`:

```rust
use ironic::json::{json, Value};
// ironic::json::json!({...})  — same as serde_json::json!
// ironic::Value               — same as serde_json::Value
```

#### `ironic::time`

Re-exports from `chrono` (requires `cron` or `logging` feature):

```rust
use ironic::time::{DateTime, Duration, Utc};
```

### Feature Modules

| Module Path | Feature | Description |
|-------------|---------|-------------|
| `ironic::auth` | `auth` | Authentication (JWT, OAuth, sessions) |
| `ironic::security` | `security` | CORS, rate limiting, security headers |
| `ironic::metrics` | `metrics` | Prometheus metrics |
| `ironic::logging` | `logging` | Structured logging |
| `ironic::telemetry` | `telemetry` | OpenTelemetry distributed tracing |
| `ironic::graphql_integration` | `graphql` | GraphQL schema builder |
| `ironic::distributed` | `distributed` | Queues, microservices, gRPC, CQRS, sagas |
| `ironic::services` | `cache`, `scheduling`, `events`, `realtime`, `sse` | Service layer (cache, SSE, events) |
| `ironic::integrations` | (any) | Third-party integrations |
| `ironic::resilience` | `resilience` | Retry, circuit breaker |
| `ironic::mcp` | `mcp` | MCP protocol server |
| `ironic::ecosystem` | `plugins`, `devtools` | Plugin ecosystem |

---

## Re-exported Crates

These crates are available through `ironic::*` so you don't need to add them to your `Cargo.toml`.

| Crate | Path | Feature |
|-------|------|---------|
| `axum` | `ironic::axum::*` | always |
| `tokio` | `ironic::tokio::*` | always (via `__private`) |
| `serde` | `ironic::serde::*` | always |
| `serde_json` | `ironic::json::*` or `ironic::__private::serde_json` | always |
| `tracing` | `ironic::tracing::*` | always |
| `tracing_subscriber` | `ironic::tracing_subscriber::*` | `logging` (default) |
| `dotenvy` | `ironic::dotenvy::*` | always |
| `tonic` | `ironic::tonic::*` | `grpc` |
| `prost` | `ironic::prost::*` | `grpc` |
| `tonic_prost` | `ironic::tonic_prost::*` | `grpc` |
| `async_graphql` | `ironic::async_graphql::*` | `graphql` |
| `garde` | `ironic::garde::*` | `validation` |

**Example: Using re-exported crates**

Instead of adding `tokio`, `serde`, `serde_json` to your `Cargo.toml`, use:

```rust
// Cargo.toml only needs:
// [dependencies]
// ironic = { workspace = true }

// In your code:
use serde::{Deserialize, Serialize};
use ironic::serde_json as json;
use ironic::tokio;
use ironic::dotenvy;

// For gRPC apps (no separate tonic dep needed):
use ironic::tonic::transport::Server;

// For GraphQL apps (no separate async-graphql dep needed):
use ironic::async_graphql::{Object, Context, Result};
```

---

## Core Types & Traits

### `Application`

The entry point for building and running your app:

```rust
#[ironic::main]
async fn main() {
    let app = Application::builder()
        .module(AppModule::definition())
        .middleware(RequestLogging::new())
        .platform(AxumAdapter::new())
        .build()
        .await
        .expect("app must build");
    app.listen("0.0.0.0:8080").await.unwrap();
}
```

**Methods:**

| Method | Description |
|--------|-------------|
| `Application::builder()` | Start building the application |
| `.module(module)` | Add a root module |
| `.module_async(f)` | Add an async module factory |
| `.middleware(m)` | Add middleware to all routes |
| `.platform(adapter)` | Set the HTTP platform adapter |
| `.override_provider(p)` | Override a DI provider for testing |
| `.build()` | Compile module graph, init providers, build app |

### `ApplicationBuilder`

Builder stages determined by the platform adapter type:

```rust
ApplicationBuilder<MissingPlatform>  // Before .platform()
ApplicationBuilder<AxumAdapter>      // After .platform()
```

### `CompiledHttpApplication`

The built application returned by `.build()`:

| Method | Description |
|--------|-------------|
| `.listen(addr)` | Start the HTTP server |
| `.into_router()` | Extract the Axum `Router` |
| `.container()` | Access the DI `Container` |
| `.shutdown()` | Graceful shutdown |
| `.layer(l)` | Add an Axum tower layer |

### `HttpError`

Rich error type with code, message, and status:

```rust
HttpError::bad_request("VALIDATION", "Invalid input");
HttpError::unauthorized("TOKEN_EXPIRED", "Token has expired");
HttpError::not_found("USER_NOT_FOUND", "User does not exist");
HttpError::internal_server_error("DB_ERROR", "Database connection failed");
```

### `RequestContext`

Per-request context available in middleware, guards, and interceptors:

| Method | Description |
|--------|-------------|
| `.request()` | Access the HTTP request |
| `.response()` | Access or set the response |
| `.route_metadata()` | Route metadata (path, methods, params) |
| `.container()` | Request-scoped DI container |
| `.extensions()` | Custom request extensions |

---

## Derive Macros

These are invoked with `#[derive(MacroName)]`:

### `#[derive(Module)]`

Defines a module in the DI graph:

```rust
#[derive(Module)]
#[module(
    imports = [OtherModule],        // modules this module depends on
    controllers = [UserController], // route controllers
    providers = [UserService],      // injectable providers
    exports = [UserService],        // providers visible to importing modules
)]
pub struct AppModule;
```

The `#[module(...)]` attribute supports:

| Attribute | Type | Description |
|-----------|------|-------------|
| `imports` | `[Type, ...]` | Other modules to import |
| `controllers` | `[Type, ...]` | Controllers in this module |
| `providers` | `[Type, ...]` | Injectable providers to register |
| `exports` | `[Type, ...]` | Providers to make visible to importers |
| `global` | `bool` | Make all exports visible globally |

### `#[derive(Injectable)]`

Registers a type as injectable in the DI container:

```rust
#[derive(Injectable)]
pub struct UserService {
    repo: Arc<UserRepository>,  // auto-injected
}
```

Supports scopes:

```rust
#[derive(Injectable)]
#[injectable(scope = Request)]  // new instance per request
pub struct RequestScopedService;
```

| Scope | Behavior |
|-------|----------|
| `Singleton` (default) | One instance for the app lifetime |
| `Transient` | New instance every injection |
| `Request` | One instance per HTTP request |

### `#[derive(OpenApiSchema)]`

Generates OpenAPI schema for a type:

```rust
#[derive(OpenApiSchema)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
}
```

### `#[derive(Merge)]`

Merge two structs of the same type (for partial updates):

```rust
#[derive(Merge)]
pub struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
}
// update.merge(existing)  — merges non-None fields
```

### `#[derive(Serializable)]`

Role-based field serialization:

```rust
#[derive(Serializable)]
pub struct User {
    pub id: u64,
    pub email: String,
    #[serialize(roles = "admin")]
    pub ssn: String,
}
```

### Other Derive Macros

| Macro | Feature | Description |
|-------|---------|-------------|
| `FromRow` | `sqlx` | SQLx row mapping |
| `PickType` | always | Pick subset of fields from a struct |
| `OmitType` | always | Omit fields from a struct |
| `PartialType` | always | Make all fields `Option` |

---

## Attribute Macros

These are invoked as `#[macro]` on items.

### Route Macros

```rust
#[controller("/users")]
#[derive(Injectable)]
pub struct UserController;

#[routes]
impl UserController {
    #[get("/")]
    async fn list(&self) -> Result<Json<Vec<User>>, HttpError> { ... }

    #[get("/:id")]
    async fn get(&self, #[param] id: u64) -> Result<Json<User>, HttpError> { ... }

    #[post]
    async fn create(&self, #[body] dto: CreateUserDto) -> Result<Json<User>, HttpError> { ... }

    #[put("/:id")]
    async fn update(&self, #[param] id: u64, #[body] dto: UpdateUserDto) -> Result<Json<User>, HttpError> { ... }

    #[delete("/:id")]
    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> { ... }

    #[head("/:id")]
    async fn head(&self, #[param] id: u64) -> Result<(), HttpError> { ... }

    #[options]
    async fn options(&self) -> Result<Json<()>, HttpError> { ... }

    #[patch("/:id")]
    async fn patch(&self, #[param] id: u64, #[body] dto: PatchUserDto) -> Result<Json<User>, HttpError> { ... }
}
```

| Macro | HTTP Method | Description |
|-------|-------------|-------------|
| `#[get(path)]` | GET | Read resource |
| `#[post(path)]` | POST | Create resource |
| `#[put(path)]` | PUT | Full update |
| `#[patch(path)]` | PATCH | Partial update |
| `#[delete(path)]` | DELETE | Delete resource |
| `#[head(path)]` | HEAD | Headers only |
| `#[options(path)]` | OPTIONS | Available methods |
| `#[controller(path)]` | — | Route prefix for all methods |

### Parameter Decorators

```rust
async fn create(
    #[body] dto: CreateUserDto,         // JSON body
    #[param] id: u64,                   // Path parameter
    #[query] filter: String,            // Query parameter
    #[header] auth: String,             // Header value
    #[cookie] session: String,          // Cookie value
    ctx: &mut RequestContext,           // Full context
) -> Result<Json<User>, HttpError> { }
```

| Decorator | Source | Description |
|-----------|--------|-------------|
| `#[body]` | Request body | JSON-deserialized body |
| `#[param]` | Path param | Named path segment |
| `#[query]` | Query string | URL query parameter |
| `#[header]` | HTTP header | Request header value |
| `#[cookie]` | Cookie | Cookie value |
| `#[form]` | Form body | URL-encoded form |
| `#[file]` | Upload | Multipart uploaded file |
| `#[raw_body]` | Raw body | Raw bytes |

### OpenAPI Attribute Macros

```rust
#[api(
    summary = "List all users",
    tag = "Users",
    security = "bearer",           // Optional auth
    deprecated = true,              // Optional
    operation_id = "listUsers",     // Optional
)]
```

```rust
#[resp(200, "Success", json = Vec<User>)]
#[resp(404, "Not found")]
async fn list(...) -> ...;
```

### Other Attribute Macros

| Macro | Feature | Description |
|-------|---------|-------------|
| `#[ironic::main]` | always | Async entry point with tokio runtime |
| `#[ironic::test]` | always | Async test with tokio runtime |
| `#[cron("0 */5 * * * *")]` | `cron` | Scheduled task |
| `#[interval("5s")]` | `scheduling` | Fixed-interval task |
| `#[cache(ttl = 60)]` | `cache` | Cache response |
| `#[cache_key]` | `cache` | Mark param as cache key |
| `#[cache_ttl]` | `cache` | Dynamic cache TTL |
| `#[timeout("30s")]` | always | Request timeout |
| `#[decorator(Name)]` | always | Custom parameter decorator |
| `#[guard]` | always | Route guard attribute |
| `#[intercept]` | always | Method interceptor |
| `#[sse]` | `sse` | SSE endpoint marker |
| `#[message_handler]` | `microservices` | Message handler |
| `#[event_handler]` | `events` | Event handler |
| `#[web_socket_gateway(path)]` | `realtime` | WebSocket endpoint |
| `#[forward_ref]` | always | Forward reference marker |

---

## Helper Macros

### `guard_fn!`

Wraps guard logic in `Box::pin(async move { ... })`:

```rust
impl Guard for JwtGuard {
    fn can_activate(&self, ctx: &mut RequestContext) -> GuardFuture {
        guard_fn!({
            // ctx is moved into the async block
            GuardDecision::Allow
        })
    }
}
```

### `intercept_fn!`

Same pattern for interceptors:

```rust
impl Interceptor for TimingInterceptor {
    fn intercept(&self, ctx: &mut RequestContext, next: InterceptorNext) -> PipelineFuture {
        intercept_fn!({
            let start = Instant::now();
            let result = next.run(ctx).await;
            tracing::info!("took {:?}", start.elapsed());
            result
        })
    }
}
```

### `create_param_decorator!`

Creates a custom parameter decorator:

```rust
create_param_decorator!(current_user, CurrentUserExtractor);
// Now usable as: async fn handler(#[decorator(current_user)] user: User) { ... }
```

---

## Application Lifecycle

### Lifecycle Hooks

Implement these traits on your module or provider to hook into the lifecycle:

| Trait | Method | When Called |
|-------|--------|-------------|
| `OnModuleInit` | `on_module_init()` | After module providers are registered |
| `OnModuleLoad` | `on_module_load()` | When module is loaded |
| `OnModuleUnload` | `on_module_unload()` | When module is unloaded |
| `OnModuleConfigure` | `on_module_configure()` | During module configuration |
| `OnModuleDestroy` | `on_module_destroy()` | During module destruction |
| `OnApplicationBootstrap` | `on_application_bootstrap()` | After app starts |
| `OnApplicationShutdown` | `on_application_shutdown()` | During graceful shutdown |
| `OnServerReady` | `on_server_ready()` | When server is accepting connections |
| `OnRequestInit` | `on_request_init()` | At start of each request |
| `OnRequestDestroy` | `on_request_destroy()` | At end of each request |
| `OnError` | `on_error()` | On unhandled error |
| `OnGuardDenied` | `on_guard_denied()` | When a guard denies access |
| `BeforeShutdown` | `before_shutdown()` | Before shutdown begins |
| `AfterShutdown` | `after_shutdown()` | After shutdown completes |

### Lifecycle Order

```
┌─────────────────────────────────────────┐
│           Application Startup            │
├─────────────────────────────────────────┤
│  1. Module graph compilation             │
│  2. Provider registration                │
│  3. on_module_init() callbacks           │
│  4. on_module_load() callbacks           │
│  5. Eager provider initialization        │
│  6. Forward reference resolution         │
│  7. on_application_bootstrap()           │
│  8. on_server_ready()                    │
│  9. HTTP server starts listening         │
├─────────────────────────────────────────┤
│          Per Request                      │
├─────────────────────────────────────────┤
│  1. on_request_init()                    │
│  2. Guard evaluation                     │
│  3. Middleware chain                     │
│  4. Interceptor chain                    │
│  5. Route handler                        │
│  6. on_request_destroy()                 │
├─────────────────────────────────────────┤
│          Graceful Shutdown                │
├─────────────────────────────────────────┤
│  1. on_application_shutdown()            │
│  2. before_shutdown() callbacks          │
│  3. Connection draining                  │
│  4. after_shutdown() callbacks           │
│  5. on_module_destroy() callbacks        │
└─────────────────────────────────────────┘
```

### Async Module Init

Modules can implement `AsyncModuleInit` for async initialization:

```rust
#[derive(Module)]
#[module(providers = [DbService])]
pub struct DatabaseModule;

#[async_trait]
impl AsyncModuleInit for DatabaseModule {
    async fn init(&self, container: &Container) -> Result<(), HttpError> {
        let pool = create_db_pool().await?;
        // Register pool in DI container
        Ok(())
    }
}
```

---

## Dependency Injection

### Container

The DI container is available after the application is built:

```rust
// From the compiled application
let container = app.container();

// From a module's init callback:
async fn init(&self, container: &Container) -> Result<(), HttpError> {
    let my_service: Arc<MyService> = container
        .resolve::<MyService>()
        .await
        .expect("MyService must be registered");
}
```

### Provider Registration

```rust
// In module definition:
#[module(providers = [UserService, UserRepository])]

// Manual registration via ProviderDefinition:
ProviderDefinition::factory::<MyService, _, _>(Scope::Singleton, vec![], || async {
    MyService::new()
})
```

### Injecting Dependencies

```rust
#[derive(Injectable)]
pub struct UserService {
    // Auto-injected by type:
    repo: Arc<UserRepository>,

    // Named dependency:
    #[inject(name = "primary")]
    cache: Arc<dyn CacheBackend>,
}
```

### Provider Scopes

| Scope | Lifetime | Use Case |
|-------|----------|----------|
| `Singleton` | Application lifetime | Caches, connection pools, config |
| `Transient` | Every injection | Value objects, DTOs |
| `Request` | Per HTTP request | Request-scoped services, DB transactions |

---

## Module System

### Module Definition

A module is the fundamental unit of organization in Ironic. Every application has at least one root module (`AppModule`). Modules can import other modules, creating a module dependency graph.

```rust
#[derive(Module)]
#[module(
    imports = [HealthModule, AuthModule],
    controllers = [UserController],
    providers = [UserService, UserRepository],
    exports = [UserService],
)]
pub struct AppModule;
```

**Key rules:**
- Each provider type can only be registered once across the entire module graph (no duplicate registrations)
- Controllers are automatically registered as providers too (they're injectable)
- Exports make providers visible to modules that import this module
- Controllers are always exported automatically
- Modules cannot have circular imports (A → B → A is detected at compile time)

### Module Discovery

When the framework compiles the module graph, it:

1. Starts from the root module passed to `.module(AppModule::definition())`
2. Recursively discovers all imported modules via `imports = [...]`
3. Builds a topological order of modules (ensuring imports are processed before dependents)
4. Validates that there are no duplicate provider registrations
5. Validates that all referenced types actually exist

The discovery process is deterministic: modules are processed in declaration order within each `imports` array.

### Module Scoping

| Aspect | Behavior |
|--------|----------|
| Provider visibility | Only visible within the module and its importers (via `exports`) |
| Controller registration | Routes are registered globally regardless of module depth |
| Lifecycle hooks | Each module's lifecycle hooks run in declaration order |
| Async init | `AsyncModuleInit` runs after all providers are registered |

### Module Graph

```
┌──────────────────────────────┐
│         AppModule             │
│  ┌────────────┐               │
│  │ Health     │ (imports)     │
│  │ Module     │               │
│  └────────────┘               │
│  ┌────────────┐               │
│  │ Auth       │ (imports)     │
│  │ Module     │               │
│  └────────────┘               │
│  ┌────────────┐               │
│  │ User       │ (imports)     │
│  │ Module     │               │
│  └────────────┘               │
│                               │
│  Providers:                   │
│  ┌──────────┐  ┌──────────┐  │
│  │ Service  │  │ Repo     │  │
│  └──────────┘  └──────────┘  │
│                               │
│  Controllers:                 │
│  ┌──────────┐                 │
│  │ ApiCtrl  │                 │
│  └──────────┘                 │
└──────────────────────────────┘
```

### Module Compilation

When `.build()` is called, the framework:

1. Discovers all modules via `imports`
2. Validates provider uniqueness
3. Compiles the module graph
4. Registers all providers in the DI container
5. Registers all controllers
6. Resolves forward references
7. Runs lifecycle hooks

---

## Routing

### Route Registration

Routes are registered by adding controller structs to a module:

```rust
#[controller("/api/users")]
pub struct UserController;

#[routes]
impl UserController {
    #[get("/:id")]
    async fn get(&self, #[param] id: u64) -> Result<Json<User>, HttpError> {
        // ...
    }
}
```

The route paths support:
- Static segments: `/api/users`
- Dynamic segments: `/:id` (captured via `#[param]`)
- Wildcards: `/*path` (captured via `#[param] path: String`)

### Route Table

Inspect routes at runtime:

```bash
ironic routes  # lists all registered routes
```

### Route Metadata

```rust
#[get("/:id")]
#[api(summary = "Get user", tag = "Users")]
#[resp(200, "Found", json = User)]
#[resp(404, "Not found")]
async fn get(&self, #[param] id: u64) -> ...;
```

---

## Middleware Stack

### Pipeline Architecture

Every HTTP request passes through a chain of middleware before reaching the controller. The pipeline is:

```
Incoming Request
    │
    ▼
┌─────────────────────┐
│ RequestTracing      │  Sets up tracing span per request
├─────────────────────┤
│ RequestLogging      │  Logs request method, path, status, duration
├─────────────────────┤
│ SecurityHeaders     │  Adds XSS, CSP, HSTS headers
├─────────────────────┤
│ RateLimit           │  Checks IP-based rate limits
├─────────────────────┤
│ CORS                │  Handles CORS preflight + headers
├─────────────────────┤
│ Custom Middleware    │  User-defined middleware (in order)
├─────────────────────┤
│ Guards              │  Authentication/authorization checks
├─────────────────────┤
│ Interceptors        │  Pre/post processing
├─────────────────────┤
│ Controller          │  Route handler
└─────────────────────┘
    │
    ▼
  Response
```

### Middleware Trait

```rust
pub trait Middleware: Send + Sync + 'static {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a>;
}
```

The `next` parameter is a boxed future that represents the rest of the pipeline. Call `next.run(context)` to continue processing. The middleware can:
- Execute code before the next middleware
- Execute code after the next middleware completes
- Short-circuit by returning a response directly without calling `next`
- Transform the request or response

### Middleware Registration Order

Middleware is executed in the order it's registered:

```rust
Application::builder()
    .middleware(MiddlewareA)     // runs first
    .middleware(MiddlewareB)     // runs second
    .middleware(MiddlewareC)     // runs third
```

For request processing: A → B → C → Controller
For response processing: Controller → C → B → A

### Built-in Middleware

### Built-in Middleware

| Middleware | Feature | Description |
|-----------|---------|-------------|
| `RequestLogging` | always | Access logs with request ID |
| `RequestTracing` | always | Tracing span per request |
| `SecurityHeadersMiddleware` | `security` | XSS, CSP, HSTS, frame-guard |
| `CorsMiddleware` | `security` | CORS configuration |
| `RateLimitMiddleware` | `security` | Per-IP rate limiting |
| `MetricsLayer` | `metrics` | Prometheus metrics |

### Custom Middleware

```rust
pub struct MyMiddleware;

impl Middleware for MyMiddleware {
    fn handle<'a>(
        &'a self,
        ctx: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            println!("before");
            let result = next.run(ctx).await;
            println!("after");
            result
        })
    }
}

// Register:
Application::builder()
    .middleware(MyMiddleware)
    .middleware(RequestLogging::new())
```

### Order of Execution

Middleware runs in registration order:

```
Request → SecurityHeaders → RateLimit → CORS → Logging → Controller
```

---

## Security

### CORS

```rust
use ironic::security::{CorsConfig, CorsMiddleware};

CorsConfig::new()
    .allowed_origins(vec!["https://app.example.com"])
    .allow_credentials(true)
```

### Rate Limiting

```rust
use ironic::security::RateLimitMiddleware;

// 100 requests per 60 seconds per IP
RateLimitMiddleware::new(100, 60)
```

### Security Headers

```rust
use ironic::security::{SecurityHeadersConfig, SecurityHeadersMiddleware};

SecurityHeadersMiddleware::new(SecurityHeadersConfig {
    xss_protection: true,
    content_type_nosniff: true,
    frame_guard: true,
    hsts_max_age: 31536000,
    ..Default::default()
})
```

### Guards (Authentication)

```rust
pub struct JwtGuard;

impl Guard for JwtGuard {
    fn can_activate(&self, ctx: &mut RequestContext) -> GuardFuture {
        guard_fn!({
            // Extract and validate token
            GuardDecision::Allow
            // or: GuardDecision::Deny(HttpError::unauthorized(...))
        })
    }
}

// Use in controller:
#[get("/protected")]
#[guard(JwtGuard)]
async fn protected(&self) -> Result<Json<()>, HttpError> {
    // Only accessible with valid JWT
}
```

### RBAC (Role-Based Access)

```rust
#[guard(RoleGuard)]
#[api(guard_meta(roles = "admin"))]
async fn admin_only(&self) -> Result<Json<()>, HttpError> {
    // Only users with "admin" role
}
```

---

## Observability

### Logging (Default)

Enabled by the `logging` feature (default). Configured in `platform/logging.rs`:

```rust
ironic::tracing_subscriber::fmt()
    .with_env_filter("info")
    .with_target(false)
    .with_file(false)
    .with_line_number(false)
    .init();
```

### JSON Logging (Production)

```rust
ironic::tracing_subscriber::fmt()
    .json()
    .with_env_filter("info")
    .init();
```

### Prometheus Metrics

Requires `metrics` feature:

```rust
use ironic::metrics::{MetricsLayer, MetricsConfig};

.platform(
    AxumAdapter::new()
        .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
)
```

Metrics are exposed at `GET /metrics`.

### Distributed Tracing (OTLP)

Requires `telemetry` feature:

```rust
use ironic::telemetry::init_tracer;

init_tracer("my-service")?;
```

---

## Database & Storage

### SQLx (SQL Databases)

```toml
ironic = { features = ["sqlx-postgres"] }
```

```rust
use sqlx::PgPool;

let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&database_url)
    .await?;

struct DbProvider {
    pool: Arc<PgPool>,
}

impl DbProvider {
    async fn query(&self) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&*self.pool)
            .await
    }
}
```

### Redis

```toml
ironic = { features = ["redis"] }
```

```rust
use redis::AsyncCommands;

let client = redis::Client::open("redis://localhost")?;
let mut conn = client.get_multiplexed_async_connection().await?;
conn.set("key", "value").await?;
```

### MongoDB

```toml
ironic = { features = ["mongodb"] }
```

```rust
use mongodb::{Client, Collection};

let client = Client::with_uri_str("mongodb://localhost:27017").await?;
let db = client.database("myapp");
let collection: Collection<User> = db.collection("users");
```

---

## Transports

### HTTP (Default)

No special feature needed:

```rust
Application::builder()
    .module(AppModule::definition())
    .platform(AxumAdapter::new())
    .build().await?
```

### gRPC

```toml
ironic = { features = ["grpc"] }
```

```rust
use ironic::tonic::transport::Server;

Server::builder()
    .add_service(MyGrpcServer::new(my_service))
    .serve(addr)
    .await?;
```

### GraphQL

```toml
ironic = { features = ["graphql"] }
```

```rust
use ironic::async_graphql::{Object, Context, Result, Schema, EmptyMutation, EmptySubscription};

#[derive(Injectable)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, _ctx: &Context<'_>) -> Result<String> {
        Ok("Hello!".into())
    }
}
```

### WebSocket

```toml
ironic = { features = ["realtime"] }
```

```rust
#[web_socket_gateway("/ws")]
pub struct ChatGateway;

#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        Ok(format!("echo: {payload}"))
    }
}
```

### SSE (Server-Sent Events)

```toml
ironic = { features = ["sse"] }
```

```rust
use ironic::services::sse::{SseRoute, SseConfig};

let (tx, stream) = SseRoute::new(SseConfig::default());
// Store tx, broadcast events
// Mount stream as SSE endpoint
```

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `ironic new <name>` | Create new HTTP project |
| `ironic new <name> --graphql` | Create new GraphQL project |
| `ironic generate app <name>` | Add HTTP microservice to monorepo |
| `ironic generate app <name> --grpc` | Add gRPC microservice |
| `ironic generate app <name> --graphql` | Add GraphQL microservice |
| `ironic generate resource <name>` | Generate CRUD module |
| `ironic generate ready-resource auth` | Generate auth module |
| `ironic start` | Run the server |
| `ironic dev` | Run with hot reload |
| `ironic build` | Build the project |
| `ironic test` | Run tests |
| `ironic openapi` | Generate OpenAPI JSON spec |
| `ironic routes` | List registered routes |
| `ironic doctor` | Debug environment |

---

## Error Code Reference

Standard error codes used across the framework:

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `VALIDATION` | Request validation failed | 400 |
| `BAD_REQUEST` | Malformed request | 400 |
| `UNAUTHORIZED` | Missing/invalid auth | 401 |
| `FORBIDDEN` | Insufficient permissions | 403 |
| `ROUTE_NOT_FOUND` | Route not found | 404 |
| `RESOURCE_NOT_FOUND` | Resource not found | 404 |
| `CONFLICT` | Resource conflict | 409 |
| `RATE_LIMITED` | Too many requests | 429 |
| `INTERNAL_ERROR` | Internal server error | 500 |
| `SERVICE_UNAVAILABLE` | Service temporarily unavailable | 503 |

---

## Generated App Structure

### HTTP App

```
src/
├── main.rs                    # Bootstrap + RequestLogging middleware
├── app.rs                     # AppModule (NestJS-style)
├── app_controller.rs          # Root controller (GET /)
├── app_service.rs             # Root service
└── platform/
    ├── mod.rs
    ├── config.rs              # listen_addr(), env vars
    └── logging.rs             # Tracing subscriber setup
```

### gRPC App

```
src/
├── main.rs                    # Tonic server + DI container
├── app.rs                     # AppModule with providers
├── app_service.rs             # Root service
├── modules/
│   └── greet/
│       ├── mod.rs
│       ├── greeter_service.rs # Tonic Greeter impl
│       └── greet_repository.rs
├── platform/
├── build.rs                   # Proto compilation
└── proto/hello.proto          # Protobuf definitions
```

### GraphQL App

```
src/
├── main.rs                    # Axum + async-graphql endpoint
├── app.rs                     # AppModule
├── app_service.rs             # Query root with #[Object]
└── platform/
```

---

## Minimum Cargo.toml

```toml
[dependencies]
ironic = { workspace = true }
```

All other dependencies (`tokio`, `serde`, `axum`, `tracing`, `dotenvy`, etc.) come transitively through `ironic` and are re-exported as `ironic::tokio`, `ironic::serde`, etc. The only exception is `tonic-prost-build` for gRPC apps (it's a build dependency that can't come from a regular library).

| App Type | Cargo.toml |
|----------|------------|
| HTTP | `ironic = { workspace = true }` |
| gRPC | `ironic = { workspace = true, features = ["grpc"] }` + `tonic-prost-build` (build-deps) |
| GraphQL | `ironic = { workspace = true, features = ["graphql"] }` |

---

## Advanced: Container Internals

### Resolution Algorithm

When `container.resolve::<T>()` is called, the container:

1. Looks up `ProviderKey::of::<T>()` in the registration map
2. If found, checks the provider's scope:
   - **Singleton**: Returns cached instance, or constructs and caches it
   - **Transient**: Constructs a new instance every time
   - **Request**: Checks request-scoped cache, or constructs and caches for the request duration
3. If not found, checks for a `ForwardRef` registration
4. If not found, returns `ResolveError::ProviderNotFound`

### Forward References

Forward references allow circular dependencies between providers:

```rust
#[derive(Module)]
#[module(providers = [ServiceA, ServiceB])]
pub struct AppModule;

#[derive(Injectable)]
pub struct ServiceA {
    #[forward_ref]
    b: Option<Arc<ServiceB>>,  // Resolved lazily after all providers are constructed
}

#[derive(Injectable)]
pub struct ServiceB {
    a: Arc<ServiceA>,
}
```

Forward refs are resolved during the `resolve_forward_refs()` phase, which runs after all eager providers are initialized but before the server starts.

### Custom Providers

```rust
use ironic::{ContainerBuilder, ProviderDefinition, Scope};

let mut builder = ContainerBuilder::new();
builder.register(ProviderDefinition::factory::<MyService, _, _>(
    Scope::Singleton,     // scope
    vec![],               // dependencies
    || async { MyService::new() },
)).expect("registration failed");

let container = builder.build();
```

### Override Providers (Testing)

```rust
// Override a provider for testing:
Application::builder()
    .module(AppModule::definition())
    .override_provider(ProviderDefinition::value(
        MockDatabaseService::new()
    ))
    .platform(AxumAdapter::new())
    .build().await?
```

---

## Testing

### Unit Tests

Test services in isolation by constructing them directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_service() {
        let repo = Arc::new(MockUserRepository::new());
        let service = UserService { repo };
        let result = service.find(1).unwrap();
        assert_eq!(result.name, "Test User");
    }
}
```

### Integration Tests

Use `TestApplication` for full HTTP testing:

```rust
use ironic::{HttpStatus, TestApplication};

#[ironic::test]
async fn test_create_user() {
    let app = TestApplication::new::<AppModule>()
        .await
        .expect("test app must build");

    let response = app.post("/users")
        .json(&serde_json::json!({"name": "Alice"}))
        .send()
        .await;

    assert_eq!(response.status(), HttpStatus::OK);
    app.shutdown().await.unwrap();
}
```

### Override Providers in Tests

```rust
#[ironic::test]
async fn test_with_mock_db() {
    let app = TestApplication::builder::<AppModule>()
        .override_provider(ProviderDefinition::value(MockDb::new()))
        .build()
        .await
        .expect("test app must build");

    // Test with mocked database...
    app.shutdown().await.unwrap();
}
```

---

## Common Patterns

### Repository Pattern

```rust
#[derive(Injectable)]
pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: u64) -> Result<User, HttpError> {
        sqlx::query_as("SELECT * FROM users WHERE id = $1")
            .bind(id as i64)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| HttpError::internal_server_error("DB_ERROR", e.to_string()))?
            .ok_or_else(|| HttpError::not_found("USER_NOT_FOUND", "User does not exist"))
    }
}
```

### Service Layer

```rust
#[derive(Injectable)]
pub struct UserService {
    repo: Arc<UserRepository>,
    email: Arc<EmailService>,
}

impl UserService {
    pub async fn register(&self, dto: CreateUserDto) -> Result<User, HttpError> {
        // Business logic
        let user = self.repo.create(dto).await?;
        self.email.send_welcome(&user).await?;
        Ok(user)
    }
}
```

### Controller Layer

```rust
#[controller("/users")]
#[derive(Injectable)]
pub struct UserController {
    service: Arc<UserService>,
}

#[routes]
impl UserController {
    #[post]
    async fn create(&self, #[body] dto: CreateUserDto) -> Result<Json<User>, HttpError> {
        Ok(Json(self.service.register(dto).await?))
    }
}
```

### Exception Filters

```rust
pub struct NotFoundFilter;

impl ExceptionFilter for NotFoundFilter {
    fn catch(&self, error: &HttpError, _ctx: &FilterContext) -> Result<Response, HttpError> {
        if error.status() == HttpStatus::NOT_FOUND {
            Ok(Response::json(HttpStatus::NOT_FOUND, &serde_json::json!({
                "error": error.code(),
                "message": error.message(),
            })))
        } else {
            Err(error.clone()) // pass through
        }
    }
}
```

### Custom Guards

```rust
pub struct AdminGuard;

impl Guard for AdminGuard {
    fn can_activate(&self, ctx: &mut RequestContext) -> GuardFuture {
        guard_fn!({
            // Check if user has admin role
            let user = ctx.extensions().get::<CurrentUser>();
            match user {
                Some(u) if u.role == "admin" => GuardDecision::Allow,
                _ => GuardDecision::Deny(HttpError::forbidden("FORBIDDEN", "Admin only")),
            }
        })
    }
}
```

### Custom Interceptors

```rust
pub struct TimingInterceptor;

impl Interceptor for TimingInterceptor {
    fn intercept(&self, ctx: &mut RequestContext, next: InterceptorNext) -> PipelineFuture {
        intercept_fn!({
            let start = std::time::Instant::now();
            let result = next.run(ctx).await;
            let duration = start.elapsed();
            tracing::info!("request took {:?}", duration);
            result
        })
    }
}
```

### Parameter Pipes (Validation/Transformation)

```rust
pub struct TrimPipe;

impl ParameterPipe for TrimPipe {
    fn transform(&self, value: ExtractedValue, _ctx: &mut RequestContext) -> PipeFuture {
        Box::pin(async move {
            if let Some(s) = value.downcast_ref::<String>() {
                Ok(Box::new(s.trim().to_string()))
            } else {
                Ok(value)
            }
        })
    }

    fn description(&self) -> &'static str { "trim" }
}

// Usage:
// async fn create(#[param] #[pipe(TrimPipe)] name: String) -> ...;
```

### Caching Responses

Requires the `cache` feature:

```rust
use ironic::prelude::*;

#[controller("/products")]
pub struct ProductController;

#[routes]
impl ProductController {
    #[get("/:id")]
    #[cache(ttl = 60)]  // Cache for 60 seconds
    async fn get(&self, #[param] id: u64) -> Result<Json<Product>, HttpError> {
        // If cached, the response is served from cache without calling this handler
        // Cache key includes the full URL path + query parameters
        self.service.find(id).await.map(Json)
    }
}
```

### Scheduled Tasks

Requires the `scheduling` or `cron` feature:

```rust
use ironic::prelude::*;
use std::sync::Arc;

pub struct ReportGenerator {
    db: Arc<PgPool>,
}

// Fixed interval (requires `scheduling` feature):
#[interval("30s")]
async fn generate_daily_report(&self) {
    let data = sqlx::query("SELECT ...").fetch_all(&*self.db).await;
    // Process and store report
}

// Cron expression (requires `cron` feature):
#[cron("0 0 * * * *")]  // Every hour
async fn hourly_cleanup(&self) {
    // Clean up old data
}
```

### Event Bus (In-process)

Requires the `events` feature:

```rust
use ironic::prelude::*;
use ironic::services::events::EventBus;

pub struct UserCreatedEvent {
    pub user_id: u64,
    pub email: String,
}

#[event_handler]
async fn on_user_created(event: Arc<UserCreatedEvent>, bus: Arc<EventBus>) {
    println!("User created: {}", event.email);
    // The event handler is auto-registered by the `#[event_handler]` macro
}

// Publishing:
bus.publish(UserCreatedEvent { user_id: 1, email: "test@example.com" }).await;
```

### Message Handlers (Microservices)

Requires the `microservices` feature:

```rust
use ironic::prelude::*;

#[message_handler("user.created")]
async fn handle_user_created(payload: serde_json::Value) -> Result<(), HttpError> {
    let user_id = payload["id"].as_u64().unwrap();
    println!("Processing user {user_id}");
    Ok(())
}
```

### WebSocket Gateway

Requires the `realtime` feature:

```rust
#[web_socket_gateway("/chat")]
pub struct ChatGateway;

#[routes]
impl ChatGateway {
    #[subscribe_message("message")]
    async fn on_message(&self, payload: String) -> Result<String, HttpError> {
        // Broadcast to all connected clients
        Ok(format!("Echo: {payload}"))
    }

    #[subscribe_message("join")]
    async fn on_join(&self, room: String) -> Result<(), HttpError> {
        println!("Client joined room: {room}");
        Ok(())
    }
}
```

### Versioning

Requires the `versioning` feature:

```rust
use ironic::prelude::*;

// Via URL prefix:
#[controller("/api/v1/users")]
pub struct UserControllerV1;

// Via header:
// Set `ironic::VersionMetadata` in the module definition
// to enable header-based version negotiation
```

### SSE (Server-Sent Events)

Requires the `sse` feature:

```rust
use ironic::prelude::*;
use ironic::services::sse::{SseRoute, SseConfig};

// Create an SSE endpoint (typically in a controller):
async fn stream_events(&self) -> Sse<SseStream> {
    let (tx, stream) = SseRoute::new(SseConfig::default());
    // Store `tx` somewhere to broadcast events
    // The `stream` is an async generator of SSE events
    stream
}

// Broadcasting events:
// tx.send(Event::default().data("message")).unwrap();
```

### Resilience (Retry + Circuit Breaker)

Requires the `resilience` feature:

```rust
use ironic::prelude::*;
use ironic::resilience::{RetryConfig, CircuitBreakerConfig};

// Retry configuration:
let retry = RetryConfig {
    max_retries: 3,
    base_delay_ms: 100,
    max_delay_ms: 5000,
    backoff: Backoff::Exponential,
};

// Circuit breaker:
let cb = CircuitBreakerConfig {
    failure_threshold: 5,
    success_threshold: 2,
    timeout_ms: 30000,
    half_open_timeout_ms: 5000,
};

// Usage with HTTP client:
// let response = retry.execute(|| client.get(url)).await?;
```

### Serverless (AWS Lambda)

Requires the `serverless` feature:

```rust
use ironic::prelude::*;

// The application can be deployed as an AWS Lambda function:
// ironic::start_lambda(|| async {
//     Application::builder()
//         .module(AppModule::definition())
//         .platform(AxumAdapter::new())
//         .build().await
// }).await;
```

### Multipart File Upload

Requires the `multipart` feature:

```rust
use ironic::prelude::*;

#[controller("/upload")]
pub struct UploadController;

#[routes]
impl UploadController {
    #[post]
    async fn upload(&self, #[file] file: UploadedFile) -> Result<Json<()>, HttpError> {
        println!("Received file: {} ({} bytes)", file.name, file.data.len());
        // Save to disk, S3, etc.
        Ok(Json(()))
    }
}
```

### CQRS Pattern

Requires the `cqrs` feature:

```rust
use ironic::prelude::*;

// Command
pub struct CreateUserCommand {
    pub name: String,
    pub email: String,
}

// Command handler
#[message_handler("command.create_user")]
async fn handle_create_user(cmd: CreateUserCommand) -> Result<(), HttpError> {
    // Validate and execute command
    println!("Creating user: {}", cmd.name);
    Ok(())
}

// Query
pub struct GetUserQuery {
    pub user_id: u64,
}

// Query handler
#[message_handler("query.get_user")]
async fn handle_get_user(query: GetUserQuery) -> Result<User, HttpError> {
    // Query read model
    Ok(User { id: query.user_id, name: "Alice".into() })
}
```

### Sagas (Distributed Transactions)

Requires the `sagas` feature:

```rust
use ironic::prelude::*;

pub struct OrderSaga;

#[saga]
impl OrderSaga {
    #[step(timeout = "30s")]
    async fn reserve_inventory(&self, order_id: u64) -> Result<(), HttpError> {
        // Reserve items in inventory
        Ok(())
    }

    #[step(timeout = "30s")]
    async fn process_payment(&self, order_id: u64) -> Result<(), HttpError> {
        // Process payment
        Ok(())
    }

    #[compensating_step(for = "process_payment")]
    async fn refund_payment(&self, order_id: u64) -> Result<(), HttpError> {
        // Refund if payment succeeded but later step failed
        Ok(())
    }

    #[step(timeout = "10s")]
    async fn send_confirmation(&self, order_id: u64) -> Result<(), HttpError> {
        // Send email confirmation
        Ok(())
    }
}
```

### Job Queues

Requires the `queues` and `redis` features:

```rust
use ironic::prelude::*;
use ironic::distributed::queues::{QueueConfig, RedisQueue};

// Configure queue
let queue = RedisQueue::new(QueueConfig {
    name: "email-notifications".into(),
    prefix: "myapp".into(),
    visibility_timeout: 60,
    max_retries: 3,
});

// Enqueue
queue.enqueue(serde_json::json!({
    "to": "user@example.com",
    "subject": "Welcome!",
})).await?;

// Process (worker)
while let Some(message) = queue.dequeue().await? {
    let payload: serde_json::Value = message.payload();
    // Process...
    queue.acknowledge(&message).await?;
}
```

---

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | Bind address |
| `SERVER_PORT` | `8080` | Listen port |
| `RUST_LOG` | `info` | Log level filter |
| `DATABASE_URL` | — | Database connection string |
| `CORS_ORIGINS` | `[]` | Allowed CORS origins |
| `RATE_LIMIT_MAX` | `100` | Max requests per minute per IP |

### Configuration via `ironic.toml`

```toml
[project]
name = "my-app"
source_root = "src"
default_module = "src/app.rs"

[generate]
module_path = "src/modules"
```

### Environment-Specific Config

```rust
use ironic::prelude::*;

#[derive(Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub redis_url: Option<String>,
}

// Load from environment:
let config = AppConfig {
    database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
    redis_url: std::env::var("REDIS_URL").ok(),
};
```

### Secret Management

```rust
use ironic::{Secret, SecretString};

pub struct AppConfig {
    pub db_password: SecretString,  // Debug output masked: "****"
}
```

---

## Best Practices

### Module Organization

```
src/
├── modules/
│   ├── users/
│   │   ├── mod.rs              # Module definition
│   │   ├── users.controller.rs # Route handlers
│   │   ├── users.service.rs    # Business logic
│   │   ├── users.repository.rs # Data access
│   │   ├── dto/                # Data transfer objects
│   │   └── entities/           # Domain models
│   └── auth/                   # Auth module (similar structure)
```

### Dependency Injection Guidelines

1. **Services** depend on repositories: `Arc<UserRepository>`
2. **Controllers** depend on services: `Arc<UserService>`
3. **Repositories** depend on infrastructure: `Arc<PgPool>`
4. **Never inject controllers** into other providers — they're route-only
5. Use `#[forward_ref]` for A→B→A circular dependencies
6. Use `Scope::Request` for request-scoped data like `CurrentUser`

### Error Handling Patterns

```rust
// Domain-specific errors:
pub enum UserError {
    NotFound,
    DuplicateEmail,
    InvalidPassword,
}

impl From<UserError> for HttpError {
    fn from(e: UserError) -> Self {
        match e {
            UserError::NotFound => HttpError::not_found("USER_NOT_FOUND", "User not found"),
            UserError::DuplicateEmail => HttpError::bad_request("DUPLICATE_EMAIL", "Email already exists"),
            UserError::InvalidPassword => HttpError::bad_request("INVALID_PASSWORD", "Password too weak"),
        }
    }
}

// In service:
fn create(&self, dto: CreateUserDto) -> Result<User, UserError> { ... }

// In controller (auto-converts via From):
async fn create(&self, #[body] dto: CreateUserDto) -> Result<Json<User>, HttpError> {
    Ok(Json(self.service.create(dto).await?))  // ? converts UserError -> HttpError
}
```

### Performance Tips

1. **Use `Scope::Singleton`** for stateless services and connection pools
2. **Avoid `Scope::Transient`** for frequently-injected services — prefer Singleton
3. **Use connection pooling**: configure `max_connections` for SQLx, Redis
4. **Enable release profile**: `lto = true`, `codegen-units = 1`, `opt-level = "z"`
5. **Use compression**: `.compression()` on the AxumAdapter
6. **Add rate limiting**: `RateLimitMiddleware::new(100, 60)` to prevent abuse
7. **Monitor with metrics**: enable `metrics` feature and scrape `/metrics`

### Security Best Practices

1. **Never hardcode secrets** — use environment variables or a vault
2. **Enable security headers**: `SecurityHeadersMiddleware`
3. **Restrict CORS origins**: specify exact origins, never `*` in production
4. **Rate limit all endpoints**: start with 100 req/min per IP
5. **Validate all input**: use `#[validate]` with `garde`
6. **Audit dependencies**: `cargo audit` regularly
7. **Use HTTPS** behind a reverse proxy (nginx, Traefik, ALB)

---

## Troubleshooting

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| `ProviderNotFound` | Service not registered in any module | Add to `#[module(providers = [...])]` |
| `CircularDependency` | A → B → A cycle | Use `#[forward_ref]` |
| `RouteNotFound` (404) | No controller registered | Add to `#[module(controllers = [...])]` |
| `OpenAPI spec empty` | `openapi` feature not enabled | Add `features = ["openapi"]` |
| `CORS errors` | Wrong origin config | Check `CORS_ORIGINS` env var |
| `Rate limit too strict` | Too few requests allowed | Increase `RATE_LIMIT_MAX` |
| `slow startup` | Many dependencies to compile | Use Docker with `--mount=type=cache` |

### Generated Code Structure

When you run `ironic new <name>`, the CLI generates:

| File | Purpose |
|------|---------|
| `Cargo.toml` | Package manifest with workspace dependency |
| `Dockerfile` | Multi-stage Alpine + musl build, scratch runtime |
| `rust-toolchain.toml` | Rust version pinning |
| `ironic.toml` | Ironic project configuration |
| `.env.example` | Environment variable template |
| `.gitignore` | Standard Rust ignores |
| `README.md` | Project documentation |
| `PRODUCTION.md` | Production readiness guide |
| `src/main.rs` | Application entry point |
| `src/app.rs` | Root AppModule |
| `src/platform/mod.rs` | Platform module declarations |
| `src/platform/config.rs` | Env config helpers (listen_addr) |
| `src/platform/logging.rs` | Tracing subscriber initialization |

### Mocking in Tests

```rust
// Mock repository for unit testing:
pub struct MockUserRepository;

impl UserRepository for MockUserRepository {
    fn find(&self, id: u64) -> Result<User, HttpError> {
        Ok(User { id, name: "Mock".into() })
    }
}

#[test]
fn test_service_with_mock() {
    let repo = Arc::new(MockUserRepository);
    let service = UserService { repo };
    let user = service.find(1).unwrap();
    assert_eq!(user.name, "Mock");
}
```

### Benchmarking

```rust
#[ironic::test]
async fn benchmark_list_users() {
    let app = TestApplication::new::<AppModule>().await.unwrap();

    let start = std::time::Instant::now();
    for _ in 0..100 {
        app.get("/users").send().await.assert_status(200);
    }
    let duration = start.elapsed();
    println!("100 requests took {:?} ({:?} per request)", duration, duration / 100);

    app.shutdown().await.unwrap();
}
```

### Debug Mode

```bash
RUST_LOG=debug ironic start     # Verbose logging
ironic doctor                    # Environment diagnostics
ironic routes                    # List all routes
ironic graph                     # Module dependency graph
```
