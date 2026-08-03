---
title: API Reference
description: The full public API surface — public modules, re-exported crates, core types & traits, and every macro.
---

# API Reference

This page is the complete public API surface of the `ironic` crate: public modules,
re-exported crates, core types & traits, derive macros, attribute macros, helper
macros, error codes, and the minimum `Cargo.toml`.

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
| `#[message]` | `microservices` | Message handler |
| `#[event]` | `events` | Event handler |
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
