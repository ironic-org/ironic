# RustFrame

> A modular, type-safe, testable application framework for building structured Rust backend systems.

RustFrame is an open-source backend application framework inspired by the architectural consistency and developer experience of NestJS, while remaining faithful to Rust's type system, ownership model, traits, explicitness, and compile-time guarantees.

RustFrame is not intended to replace Axum, Tokio, Tower, Hyper, SQLx, or other established Rust libraries. Instead, it provides an opinionated application layer on top of them.

---

## Table of Contents

1. [Vision](#vision)
2. [Goals](#goals)
3. [Non-Goals](#non-goals)
4. [Core Principles](#core-principles)
5. [Technology Stack](#technology-stack)
6. [High-Level Architecture](#high-level-architecture)
7. [Workspace Structure](#workspace-structure)
8. [Application Bootstrap](#application-bootstrap)
9. [Module System](#module-system)
10. [Dependency Injection](#dependency-injection)
11. [Controllers](#controllers)
12. [Routing](#routing)
13. [Providers and Services](#providers-and-services)
14. [Execution Context](#execution-context)
15. [Request Lifecycle](#request-lifecycle)
16. [Middleware](#middleware)
17. [Guards](#guards)
18. [Pipes and Validation](#pipes-and-validation)
19. [Interceptors](#interceptors)
20. [Exception Handling](#exception-handling)
21. [Configuration](#configuration)
22. [Logging and Observability](#logging-and-observability)
23. [Lifecycle Hooks](#lifecycle-hooks)
24. [Platform Adapters](#platform-adapters)
25. [OpenAPI](#openapi)
26. [Database Integrations](#database-integrations)
27. [Authentication and Authorization](#authentication-and-authorization)
28. [Caching](#caching)
29. [Scheduling](#scheduling)
30. [Queues and Background Jobs](#queues-and-background-jobs)
31. [Events and CQRS](#events-and-cqrs)
32. [WebSockets and SSE](#websockets-and-sse)
33. [Microservices](#microservices)
34. [GraphQL](#graphql)
35. [Testing](#testing)
36. [CLI](#cli)
37. [Code Generation](#code-generation)
38. [Project Configuration](#project-configuration)
39. [Developer Tools](#developer-tools)
40. [Error Model](#error-model)
41. [Security](#security)
42. [Performance](#performance)
43. [Documentation Strategy](#documentation-strategy)
44. [Release Roadmap](#release-roadmap)
45. [Repository Milestones](#repository-milestones)
46. [RFC Process](#rfc-process)
47. [Contribution Guidelines](#contribution-guidelines)
48. [Example Application](#example-application)
49. [Initial Public API](#initial-public-api)
50. [Implementation Order](#implementation-order)

---

# Vision

RustFrame aims to make large Rust backend applications easier to structure, test, extend, and maintain.

The framework should provide a consistent architecture for:

- REST APIs
- Modular monoliths
- Microservices
- Background workers
- Event-driven systems
- WebSocket applications
- Command-line applications
- Internal services
- Enterprise backend platforms

The intended developer experience is:

```rust
use rustframe::prelude::*;

#[derive(Module)]
#[module(
    imports = [UsersModule],
)]
pub struct AppModule;

#[rustframe::main]
async fn main() -> FrameworkResult<()> {
    RustFrame::create::<AppModule>()
        .global_interceptor(TracingInterceptor)
        .global_error_handler(DefaultErrorHandler)
        .listen("0.0.0.0:3000")
        .await
}
```

The framework should feel structured and productive without hiding Rust.

---

# Goals

RustFrame should:

- Provide a consistent modular application architecture.
- Offer compile-time metadata wherever possible.
- Support dependency injection without excessive runtime magic.
- Make large applications easier to organize.
- Provide a first-class CLI similar in purpose to Nest CLI.
- Support code generation for common backend components.
- Offer testing utilities as a core feature.
- Work naturally with Tokio, Axum, Tower, Hyper, Serde, SQLx, and Tracing.
- Allow developers to escape to the underlying platform when required.
- Keep optional integrations outside the core package.
- Support open-source community extensions.

---

# Non-Goals

RustFrame should not:

- Implement a new async runtime.
- Implement a new HTTP protocol stack.
- Replace Axum, Hyper, or Tower.
- Force a specific ORM or database library.
- Hide normal Rust types and traits.
- Make reflection-heavy runtime behavior the primary model.
- Require string-based dependency tokens for normal use.
- Reimplement every external library.
- Include GraphQL, queues, CQRS, and microservices in the first release.

---

# Core Principles

## Rust-First Design

NestJS can inspire the architecture, but RustFrame must not imitate TypeScript patterns that conflict with Rust.

RustFrame should prioritize:

- Traits
- Structs
- Enums
- Strong typing
- Explicit constructors
- Compile-time code generation
- Ownership and borrowing
- `Result<T, E>`
- `Arc<T>` for shared ownership

## Compile-Time Over Runtime

Macros should generate explicit implementations.

User code:

```rust
#[derive(Injectable)]
pub struct UsersService {
    repository: Arc<UserRepository>,
}
```

Generated behavior should conceptually resemble:

```rust
impl Provider for UsersService {
    fn create(container: &Container) -> Result<Self, ResolveError> {
        Ok(Self {
            repository: container.resolve::<UserRepository>()?,
        })
    }
}
```

## Minimal Core

The core should contain only:

- Application bootstrap
- Module system
- Dependency injection
- Provider lifecycle
- Controller metadata
- Request pipeline abstractions
- Platform adapter contracts

Everything else should be optional.

## Escape Hatches

Developers should be able to use:

- Axum extractors
- Tower layers
- Tokio tasks
- Hyper request and response types
- Native SQLx pools
- Third-party middleware

## Predictable Request Lifecycle

Framework behavior must be documented and deterministic.

---

# Technology Stack

Recommended initial stack:

| Area | Recommended Library |
|---|---|
| Async runtime | Tokio |
| HTTP framework | Axum |
| Middleware | Tower |
| HTTP implementation | Hyper |
| Serialization | Serde |
| Validation | Garde or Validator |
| Error definitions | Thiserror |
| CLI | Clap |
| Interactive CLI | Dialoguer |
| Terminal output | Console and Indicatif |
| Procedural macros | Syn and Quote |
| Generated formatting | Prettyplease |
| Configuration | Figment or custom typed layer |
| Logging | Tracing |
| Metrics | Metrics crate |
| Telemetry | OpenTelemetry |
| OpenAPI | Utoipa integration |
| Database example | SQLx |
| GraphQL | async-graphql |
| gRPC | Tonic |
| WebSocket | Axum WebSocket or tokio-tungstenite |

---

# High-Level Architecture

```text
Application Code
      |
      v
RustFrame User API
      |
      v
Generated Metadata and Trait Implementations
      |
      v
Framework Kernel
      |
      v
Platform Adapter
      |
      v
Axum / Tower / Hyper / Tokio
```

The framework should be divided into four layers.

## User-Facing API

This includes:

- `#[module]`
- `#[controller]`
- `#[get]`
- `#[post]`
- `#[derive(Injectable)]`
- `#[use_guard]`
- `#[use_interceptor]`

## Generated Metadata

Macros should generate:

- Route definitions
- Controller definitions
- Provider definitions
- Module definitions
- Extractor adapters
- OpenAPI metadata

## Framework Kernel

The kernel handles:

- Module graph compilation
- Dependency graph validation
- Provider registration
- Provider instantiation
- Lifecycle hooks
- Route registration
- Global pipeline components
- Shutdown
- Testing overrides

## Platform Adapter

The first adapter should target Axum.

Later adapters may target:

- Actix Web
- Poem
- Salvo
- Hyper directly
- Serverless runtimes

---

# Workspace Structure

```text
rustframe/
├── Cargo.toml
├── crates/
│   ├── rustframe/
│   ├── rustframe-core/
│   ├── rustframe-common/
│   ├── rustframe-di/
│   ├── rustframe-macros/
│   ├── rustframe-http/
│   ├── rustframe-platform/
│   ├── rustframe-platform-axum/
│   ├── rustframe-config/
│   ├── rustframe-validation/
│   ├── rustframe-testing/
│   ├── rustframe-openapi/
│   ├── rustframe-cli/
│   ├── rustframe-health/
│   ├── rustframe-cache/
│   ├── rustframe-schedule/
│   ├── rustframe-websocket/
│   ├── rustframe-microservices/
│   ├── rustframe-cqrs/
│   └── rustframe-telemetry/
├── examples/
│   ├── hello-world/
│   ├── rest-api/
│   ├── authentication/
│   ├── database-sqlx/
│   └── complete-api/
├── docs/
├── rfcs/
├── benchmarks/
└── tests/
```

## Crate Responsibilities

### `rustframe`

Public facade and prelude.

```rust
use rustframe::prelude::*;
```

### `rustframe-core`

Contains:

- Application builder
- Application context
- Module compiler
- Lifecycle system
- Global framework configuration

### `rustframe-common`

Contains:

- Shared types
- Metadata structures
- Common error types
- Transport-neutral abstractions

### `rustframe-di`

Contains:

- Container
- Provider registration
- Provider resolution
- Scope handling
- Dependency graph validation

### `rustframe-macros`

Contains procedural macros.

### `rustframe-http`

Contains:

- HTTP method abstraction
- Route definitions
- Request and response abstractions
- Extractor contracts

### `rustframe-platform`

Contains platform adapter traits.

### `rustframe-platform-axum`

Converts framework definitions into an Axum router.

### `rustframe-testing`

Contains:

- Test module builder
- Provider overrides
- Test application
- Test client

### `rustframe-cli`

Contains:

- Project generator
- Code generators
- Build and start commands
- Route inspector
- Dependency graph tools
- Doctor command

---

# Application Bootstrap

The framework should support an explicit builder API first.

```rust
#[tokio::main]
async fn main() -> Result<(), FrameworkError> {
    let app = FrameworkApplication::builder()
        .module(AppModule::definition())
        .build()
        .await?;

    app.listen("127.0.0.1:3000").await
}
```

After the kernel is stable, provide macro sugar:

```rust
#[rustframe::main]
async fn main() -> FrameworkResult<()> {
    RustFrame::create::<AppModule>()
        .listen("127.0.0.1:3000")
        .await
}
```

## Bootstrap Responsibilities

1. Load configuration.
2. Compile module graph.
3. Validate provider graph.
4. Instantiate singleton providers.
5. Run initialization hooks.
6. Build controller metadata.
7. Build the platform router.
8. Start the server.
9. Listen for shutdown signals.
10. Run shutdown hooks.

---

# Module System

Modules group related providers, controllers, imports, and exports.

```rust
#[derive(Module)]
#[module(
    imports = [DatabaseModule, ConfigModule],
    controllers = [UsersController],
    providers = [UsersService, UserRepository],
    exports = [UsersService],
)]
pub struct UsersModule;
```

## Module Metadata

```rust
pub struct ModuleMetadata {
    pub name: &'static str,
    pub imports: Vec<ModuleDefinition>,
    pub controllers: Vec<ControllerDefinition>,
    pub providers: Vec<ProviderDefinition>,
    pub exports: Vec<ProviderToken>,
    pub global: bool,
}
```

## Initial Module Features

- Imports
- Providers
- Controllers
- Exports
- Global modules
- Duplicate detection
- Circular import detection
- Module initialization order

## Future Module Features

- Dynamic modules
- Async module configuration
- Lazy modules
- Feature modules
- Conditional modules
- Module re-exports

## Module Compilation

The module compiler should:

1. Start with the root module.
2. Recursively visit imports.
3. Detect cycles.
4. Deduplicate modules.
5. Register providers.
6. Validate exports.
7. Build controller list.
8. Produce a compiled application graph.

---

# Dependency Injection

Dependency injection is one of the most important parts of RustFrame.

## Initial Scopes

```rust
pub enum Scope {
    Singleton,
    Transient,
}
```

Request scope should be added later.

## Provider Trait

```rust
pub trait Provider: Send + Sync + 'static {
    fn create(container: &Container) -> Result<Self, ResolveError>
    where
        Self: Sized;
}
```

## Container Structure

```rust
pub struct Container {
    registrations: HashMap<TypeId, ProviderRegistration>,
    singletons: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}
```

## Resolution API

```rust
let service = container.resolve::<UsersService>()?;
```

## Provider Registration

```rust
container.register::<UsersService>();
container.register::<UserRepository>();
```

## Trait-Based Dependencies

Trait-object injection should be explicit.

```rust
pub trait UserRepository: Send + Sync {
    fn find_by_id(&self, id: Uuid) -> RepositoryFuture<User>;
}
```

A wrapper token can be used:

```rust
pub struct UserRepositoryProvider(
    pub Arc<dyn UserRepository>,
);
```

Or provide a binding API:

```rust
container
    .bind::<dyn UserRepository>()
    .to::<SqlUserRepository>();
```

## Required DI Features

- Singleton providers
- Transient providers
- Constructor injection
- Async factory providers
- Value providers
- Alias providers
- Optional dependencies
- Circular dependency detection
- Provider override for testing
- Clear resolution errors

## Avoid

- String tokens as the primary model
- Hidden field mutation
- Runtime reflection dependence
- Implicit global state

---

# Controllers

Controllers receive transport requests and delegate business logic to services.

```rust
#[controller("/users")]
pub struct UsersController {
    service: Arc<UsersService>,
}
```

Controller constructors should be generated through DI.

```rust
#[derive(Injectable)]
pub struct UsersController {
    service: Arc<UsersService>,
}
```

Controllers should remain thin.

Good controller responsibilities:

- Parse request input
- Invoke application service
- Convert result into response
- Attach route metadata

Controllers should not contain:

- Database queries
- Password hashing implementation
- Long business workflows
- Low-level infrastructure logic

---

# Routing

```rust
#[routes]
impl UsersController {
    #[get("/:id")]
    async fn find_one(
        &self,
        #[param] id: Uuid,
    ) -> Result<Json<UserResponse>, AppError> {
        self.service.find_one(id).await.map(Json)
    }

    #[post("/")]
    async fn create(
        &self,
        #[body] dto: Validated<CreateUserDto>,
    ) -> Result<Json<UserResponse>, AppError> {
        self.service.create(dto.into_inner()).await.map(Json)
    }
}
```

## Initial HTTP Methods

- GET
- POST
- PUT
- PATCH
- DELETE
- HEAD
- OPTIONS

## Initial Parameter Sources

- Path parameters
- Query parameters
- JSON body
- Headers
- Cookies
- Request extensions
- Shared context

## Route Metadata

```rust
pub struct RouteDefinition {
    pub method: HttpMethod,
    pub path: &'static str,
    pub handler: HandlerDefinition,
    pub guards: Vec<GuardDefinition>,
    pub interceptors: Vec<InterceptorDefinition>,
    pub metadata: MetadataMap,
}
```

## Route Features for Later Releases

- API versioning
- Host routing
- Subdomain routing
- File responses
- Streaming responses
- Redirects
- Route groups
- Deprecated routes

---

# Providers and Services

A provider is any object managed by the DI container.

Examples:

- Services
- Repositories
- Configuration objects
- Cache clients
- Database pools
- Message clients
- Factories
- Adapters

```rust
#[derive(Injectable)]
pub struct UsersService {
    repository: Arc<UserRepositoryProvider>,
}
```

A service should contain application or business logic.

```rust
impl UsersService {
    pub async fn find_one(&self, id: Uuid) -> Result<UserResponse, AppError> {
        let user = self.repository.0.find_by_id(id).await?;
        Ok(user.into())
    }
}
```

---

# Execution Context

Execution context provides metadata and transport information to guards, interceptors, filters, and custom extractors.

```rust
pub struct ExecutionContext<'a> {
    pub handler: &'a HandlerMetadata,
    pub controller: &'a ControllerMetadata,
    pub container: &'a Container,
    pub transport: TransportContext<'a>,
    pub extensions: &'a Extensions,
}
```

```rust
pub enum TransportContext<'a> {
    Http(HttpContext<'a>),
    WebSocket(WebSocketContext<'a>),
    Rpc(RpcContext<'a>),
    GraphQl(GraphQlContext<'a>),
}
```

The first release may only implement `Http`.

---

# Request Lifecycle

The request pipeline should be documented as:

```text
Incoming Request
      |
      v
Platform Middleware
      |
      v
Framework Middleware
      |
      v
Guards
      |
      v
Interceptors: Before
      |
      v
Parameter Extraction
      |
      v
Validation and Transformation
      |
      v
Controller Handler
      |
      v
Interceptors: After
      |
      v
Exception Mapping
      |
      v
Outgoing Response
```

The exact behavior must be stable and covered by tests.

---

# Middleware

Middleware runs before route guards and handlers.

```rust
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(
        &self,
        request: FrameworkRequest,
        next: Next,
    ) -> Result<FrameworkResponse, FrameworkError>;
}
```

Common use cases:

- Request IDs
- Request logging
- Tenant resolution
- Locale selection
- Correlation IDs
- Authentication token parsing
- Metrics

Tower middleware must remain supported.

```rust
app.layer(TraceLayer::new_for_http());
```

---

# Guards

Guards decide whether a request may invoke a handler.

```rust
#[async_trait]
pub trait Guard: Send + Sync {
    async fn can_activate(
        &self,
        context: &ExecutionContext<'_>,
    ) -> Result<bool, GuardError>;
}
```

Usage:

```rust
#[get("/profile")]
#[use_guard(JwtAuthGuard)]
async fn profile(
    &self,
    #[current_user] user: AuthenticatedUser,
) -> Json<UserProfile> {
    Json(user.into())
}
```

Common guards:

- JWT authentication
- API key authentication
- Role guard
- Permission guard
- Tenant guard
- Resource ownership guard
- Feature flag guard

---

# Pipes and Validation

Pipes transform and validate handler parameters.

Possible built-ins:

- `ParseInt`
- `ParseFloat`
- `ParseBool`
- `ParseUuid`
- `ParseEnum`
- `ParseDate`
- `DefaultValue`
- `Validated<T>`

Example:

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}
```

```rust
#[post("/")]
async fn create(
    &self,
    #[body] dto: Validated<CreateUserDto>,
) -> Result<Json<UserResponse>, AppError> {
    self.service.create(dto.into_inner()).await.map(Json)
}
```

## Validation Error Format

```json
{
  "status": 422,
  "code": "VALIDATION_ERROR",
  "message": "Request validation failed",
  "errors": [
    {
      "field": "email",
      "message": "Must be a valid email address"
    }
  ]
}
```

---

# Interceptors

Interceptors wrap handler execution.

```rust
#[async_trait]
pub trait Interceptor: Send + Sync {
    async fn intercept(
        &self,
        context: &ExecutionContext<'_>,
        next: CallHandler,
    ) -> Result<FrameworkResponse, FrameworkError>;
}
```

Common uses:

- Response envelopes
- Logging
- Request timing
- Metrics
- Caching
- Transactions
- Timeout handling
- Audit logging
- Response serialization

Example:

```rust
#[use_interceptor(ResponseEnvelopeInterceptor)]
async fn find_all(&self) -> Vec<UserResponse> {
    self.service.find_all().await
}
```

Response:

```json
{
  "success": true,
  "data": []
}
```

---

# Exception Handling

RustFrame should use `Result<T, E>` as the main error model.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("User not found")]
    UserNotFound,

    #[error("Access denied")]
    Forbidden,

    #[error("Database error")]
    Database(#[from] sqlx::Error),
}
```

```rust
pub trait IntoFrameworkResponse {
    fn into_response(self) -> FrameworkResponse;
}
```

```rust
impl IntoFrameworkResponse for AppError {
    fn into_response(self) -> FrameworkResponse {
        match self {
            AppError::UserNotFound => {
                FrameworkResponse::not_found("USER_NOT_FOUND", self.to_string())
            }
            AppError::Forbidden => {
                FrameworkResponse::forbidden("FORBIDDEN", self.to_string())
            }
            AppError::Database(_) => {
                FrameworkResponse::internal_error(
                    "INTERNAL_ERROR",
                    "An internal error occurred",
                )
            }
        }
    }
}
```

## Filter Support

Later releases may support typed filters:

```rust
#[exception_filter(UserNotFound)]
impl ExceptionFilter<UserNotFound> for UserNotFoundFilter {
    fn catch(
        &self,
        error: UserNotFound,
        context: &ExecutionContext<'_>,
    ) -> FrameworkResponse {
        FrameworkResponse::not_found("USER_NOT_FOUND", error.to_string())
    }
}
```

---

# Configuration

Configuration should be typed.

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct DatabaseConfig {
    pub url: String,

    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}
```

Potential sources:

- Environment variables
- `.env`
- TOML
- YAML
- JSON
- Secret manager adapters

Example:

```rust
let config = ConfigModule::builder()
    .env_file(".env")
    .load::<AppConfig>()?;
```

Features:

- Namespaced configuration
- Validation
- Profiles
- Default values
- Secret redaction
- Test overrides

---

# Logging and Observability

Use `tracing` as the default logging and instrumentation layer.

Features:

- Structured logs
- Request IDs
- Correlation IDs
- Trace IDs
- Span propagation
- Startup diagnostics
- Route timing
- Provider initialization timing
- Graceful shutdown logging

Example:

```rust
RustFrame::create::<AppModule>()
    .logger(TracingLogger::json())
    .listen("0.0.0.0:3000")
    .await?;
```

Optional telemetry package:

```text
rustframe-telemetry
```

It may support:

- OpenTelemetry traces
- Metrics export
- OTLP
- Prometheus
- Service metadata

---

# Lifecycle Hooks

```rust
#[async_trait]
pub trait OnModuleInit {
    async fn on_module_init(&self) -> Result<(), FrameworkError>;
}

#[async_trait]
pub trait OnApplicationBootstrap {
    async fn on_application_bootstrap(&self) -> Result<(), FrameworkError>;
}

#[async_trait]
pub trait OnModuleDestroy {
    async fn on_module_destroy(&self) -> Result<(), FrameworkError>;
}

#[async_trait]
pub trait OnApplicationShutdown {
    async fn on_application_shutdown(
        &self,
        signal: ShutdownSignal,
    ) -> Result<(), FrameworkError>;
}
```

Use cases:

- Database connection setup
- Cache warm-up
- Message subscriptions
- Background task startup
- Resource cleanup
- Telemetry flushing

---

# Platform Adapters

The platform-neutral trait may look like:

```rust
#[async_trait]
pub trait HttpPlatformAdapter {
    async fn register_route(
        &mut self,
        route: RouteDefinition,
    ) -> Result<(), PlatformError>;

    async fn register_middleware(
        &mut self,
        middleware: MiddlewareDefinition,
    ) -> Result<(), PlatformError>;

    async fn listen(
        self,
        address: SocketAddr,
    ) -> Result<(), PlatformError>;
}
```

## Axum Adapter

The first adapter should:

- Convert route metadata into Axum routes
- Convert extractors
- Attach middleware
- Convert framework responses
- Support Tower layers
- Support graceful shutdown

## Escape Hatch

```rust
app.with_axum_router(|router| {
    router.route("/special", axum::routing::get(custom_handler))
});
```

---

# OpenAPI

Optional package:

```text
rustframe-openapi
```

Features:

- Automatic route discovery
- DTO schema generation
- Response schema generation
- Tags
- Operation IDs
- Authentication schemes
- Examples
- JSON export
- Swagger UI

Example:

```rust
#[derive(Deserialize, Validate, OpenApiSchema)]
pub struct CreateUserDto {
    pub email: String,
    pub password: String,
}
```

```rust
RustFrame::create::<AppModule>()
    .openapi(OpenApiConfig::new("User API", "1.0.0"))
    .swagger_ui("/docs")
    .listen("0.0.0.0:3000")
    .await?;
```

---

# Database Integrations

The core should not require an ORM.

Suggested packages:

```text
rustframe-sqlx
rustframe-seaorm
rustframe-diesel
rustframe-mongodb
rustframe-redis
```

Example:

```rust
#[derive(Injectable)]
pub struct UserRepository {
    pool: PgPool,
}
```

Potential features:

- Connection pool registration
- Migration helpers
- Transaction interceptor
- Health checks
- Test database helpers

Transaction example:

```rust
#[transactional]
async fn create_order(
    &self,
    command: CreateOrder,
) -> Result<Order, AppError> {
    // ...
}
```

---

# Authentication and Authorization

Suggested packages:

```text
rustframe-auth
rustframe-jwt
rustframe-session
rustframe-oauth
rustframe-rate-limit
```

Authentication should use guards and request extractors.

```rust
#[get("/profile")]
#[use_guard(JwtAuthGuard)]
async fn profile(
    &self,
    #[current_user] user: CurrentUser,
) -> Json<UserProfile> {
    Json(user.into())
}
```

Authorization metadata:

```rust
#[roles("admin")]
#[permissions("users.read")]
#[get("/:id")]
async fn find_one(...) {}
```

Security implementations should wrap trusted crates rather than implement cryptography directly.

---

# Caching

Optional package:

```text
rustframe-cache
```

Features:

- In-memory cache
- Redis adapter
- TTL
- Cache interceptor
- Cache invalidation
- Custom cache keys
- Distributed locking
- Stampede protection

Example:

```rust
#[get("/:id")]
#[cache(ttl = "60s", key = "user:{id}")]
async fn find_one(...) {}
```

---

# Scheduling

Optional package:

```text
rustframe-schedule
```

Features:

- Cron jobs
- Fixed intervals
- Delayed tasks
- Named jobs
- Concurrency control
- Distributed locks
- Retry policies

Example:

```rust
#[cron("0 */5 * * * *")]
async fn refresh_cache(&self) -> Result<(), AppError> {
    Ok(())
}
```

---

# Queues and Background Jobs

Possible adapters:

- Redis
- RabbitMQ
- NATS JetStream
- Kafka
- PostgreSQL
- Amazon SQS

Example:

```rust
#[processor("email")]
pub struct EmailProcessor;

#[jobs]
impl EmailProcessor {
    #[job("welcome-email")]
    async fn welcome(
        &self,
        job: Job<WelcomeEmailPayload>,
    ) -> Result<(), JobError> {
        Ok(())
    }
}
```

Features:

- Retries
- Backoff
- Dead-letter queue
- Delayed jobs
- Priority
- Idempotency
- Progress
- Concurrency

---

# Events and CQRS

## Event Bus

```rust
#[event_handler(UserCreated)]
async fn on_user_created(
    &self,
    event: UserCreated,
) -> Result<(), AppError> {
    Ok(())
}
```

```rust
event_bus.emit(UserCreated { user_id }).await?;
```

## CQRS Package

```text
rustframe-cqrs
```

Features:

- Command bus
- Query bus
- Event bus
- Command handlers
- Query handlers
- Event handlers
- Aggregates
- Sagas

Example:

```rust
#[derive(Command)]
pub struct CreateOrder {
    pub customer_id: Uuid,
}

#[command_handler(CreateOrder)]
impl CreateOrderHandler {
    async fn execute(
        &self,
        command: CreateOrder,
    ) -> Result<OrderId, AppError> {
        todo!()
    }
}
```

---

# WebSockets and SSE

## WebSockets

Optional package:

```text
rustframe-websocket
```

```rust
#[websocket_gateway("/chat")]
pub struct ChatGateway;

#[gateway]
impl ChatGateway {
    #[on_connect]
    async fn connected(&self, socket: Socket) {}

    #[subscribe("message")]
    async fn message(
        &self,
        payload: ChatMessage,
    ) -> Result<ChatMessageResponse, WsError> {
        todo!()
    }
}
```

Features:

- Connection lifecycle
- Message handlers
- Guards
- Interceptors
- Pipes
- Exception filters
- Rooms
- Broadcast
- Per-connection state

## Server-Sent Events

```rust
#[sse("/events")]
async fn events(
    &self,
) -> impl Stream<Item = Result<SseEvent, FrameworkError>> {
    todo!()
}
```

Features:

- Event IDs
- Event types
- Retry duration
- Heartbeats
- Disconnect cleanup

---

# Microservices

Optional package:

```text
rustframe-microservices
```

Transports may include:

- TCP
- Redis
- NATS
- MQTT
- RabbitMQ
- Kafka
- gRPC

```rust
#[message_controller]
pub struct MathController;

#[message_handlers]
impl MathController {
    #[message_pattern("sum")]
    async fn sum(
        &self,
        #[payload] numbers: Vec<i32>,
    ) -> i32 {
        numbers.into_iter().sum()
    }

    #[event_pattern("user.created")]
    async fn user_created(
        &self,
        #[payload] event: UserCreated,
    ) -> Result<(), AppError> {
        Ok(())
    }
}
```

Features:

- Request-response messaging
- Event messaging
- Correlation IDs
- Timeouts
- Retries
- Serialization
- Raw driver access
- Hybrid HTTP and microservice applications

---

# GraphQL

Optional package:

```text
rustframe-graphql
```

The recommended first implementation should wrap `async-graphql`.

Potential features:

- Queries
- Mutations
- Subscriptions
- Resolvers
- Field resolvers
- Scalars
- Interfaces
- Unions
- Data loaders
- Guards
- Interceptors

GraphQL should not be part of the MVP.

---

# Testing

Testing must be a core feature.

## Testing Module

```rust
let module = TestModule::builder::<UsersModule>()
    .override_provider::<UserRepository>(
        MockUserRepository::new(),
    )
    .compile()
    .await?;

let service = module.resolve::<UsersService>()?;
```

## Test Application

```rust
let app = TestApplication::new::<AppModule>().await?;

let response = app
    .post("/users")
    .json(&CreateUserRequest {
        email: "test@example.com".into(),
        password: "secure-password".into(),
    })
    .send()
    .await;

response.assert_status(201);
```

## Required Test Coverage

- Module imports
- Module exports
- Provider registration
- Singleton scope
- Transient scope
- Circular dependencies
- Missing dependencies
- Route registration
- Middleware order
- Guard order
- Interceptor order
- Validation errors
- Error mapping
- Lifecycle hooks
- Graceful shutdown
- Macro compile failures

## Macro Tests

Use compile-pass and compile-fail tests.

---

# CLI

The CLI should be a first-class component.

Installation:

```bash
cargo install rustframe-cli
```

Main command:

```bash
rustframe
```

## Core Commands

```bash
rustframe new <project-name>
rustframe start
rustframe start --watch
rustframe build
rustframe test
rustframe doctor
rustframe info
rustframe routes
rustframe graph
rustframe add <integration>
rustframe generate <resource>
```

Aliases:

```bash
rustframe g module users
rustframe g controller users
rustframe g service users
rustframe g resource products
```

## CLI Architecture

```text
rustframe-cli/
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── commands/
│   │   ├── new.rs
│   │   ├── start.rs
│   │   ├── build.rs
│   │   ├── test.rs
│   │   ├── generate.rs
│   │   ├── add.rs
│   │   ├── routes.rs
│   │   ├── graph.rs
│   │   ├── doctor.rs
│   │   └── info.rs
│   ├── generators/
│   │   ├── module.rs
│   │   ├── controller.rs
│   │   ├── service.rs
│   │   ├── repository.rs
│   │   ├── middleware.rs
│   │   ├── guard.rs
│   │   ├── interceptor.rs
│   │   └── resource.rs
│   ├── templates/
│   ├── project/
│   │   ├── cargo_toml.rs
│   │   ├── source_editor.rs
│   │   └── module_registry.rs
│   └── error.rs
```

## Recommended CLI Dependencies

```toml
clap = { version = "4", features = ["derive"] }
dialoguer = "0.11"
console = "0.15"
indicatif = "0.17"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
cargo_metadata = "0.18"
thiserror = "2"
include_dir = "0.7"
syn = { version = "2", features = ["full"] }
quote = "1"
prettyplease = "0.2"
```

## CLI Design Rule

The core framework must work without the CLI.

```bash
cargo run
cargo build
cargo test
```

The CLI should orchestrate and generate, not define runtime behavior.

---

# Code Generation

## Supported Generators

```bash
rustframe generate module users
rustframe generate controller users
rustframe generate service users
rustframe generate repository users
rustframe generate provider cache
rustframe generate middleware logger
rustframe generate guard auth
rustframe generate interceptor response
rustframe generate filter exception
rustframe generate dto create-user
rustframe generate entity user
rustframe generate resource products
rustframe generate gateway chat
rustframe generate command create-order
rustframe generate query get-order
```

## Aliases

| Generator | Alias |
|---|---|
| module | mo |
| controller | co |
| service | s |
| repository | repo |
| provider | pr |
| middleware | mi |
| guard | gu |
| interceptor | in |
| filter | f |
| resource | res |
| gateway | ga |

## Resource Generator

```bash
rustframe generate resource products
```

Interactive prompts:

```text
Which transport do you want?
> REST API
  GraphQL
  Microservice
  WebSocket

Generate CRUD operations?
> Yes
  No

Which database integration?
> None
  SQLx
  SeaORM
  Diesel
```

Generated structure:

```text
src/modules/products/
├── mod.rs
├── module.rs
├── controller.rs
├── service.rs
├── repository.rs
├── dto/
│   ├── mod.rs
│   ├── create_product.rs
│   └── update_product.rs
├── entities/
│   ├── mod.rs
│   └── product.rs
└── tests/
    ├── controller_test.rs
    └── service_test.rs
```

The generator should automatically register the module in its parent module when safe.

## Source Modification

Avoid fragile text replacement.

Use syntax-aware editing where possible:

- `syn`
- `quote`
- `prettyplease`

When automatic editing is unsafe, print a clear manual registration instruction.

---

# Project Configuration

Project-level configuration:

```toml
# rustframe.toml

[project]
name = "ecommerce-api"
source_root = "src"
default_module = "src/app.rs"

[generate]
spec = true
flat = false
module_path = "src/modules"

[server]
host = "127.0.0.1"
port = 3000

[platform]
adapter = "axum"
```

Possible settings:

- Source root
- Default module file
- Generator destination
- Flat or nested generation
- Test generation
- Default adapter
- Default validation package
- Default database integration
- Server port

---

# Developer Tools

## Route Inspector

```bash
rustframe routes
```

Example:

```text
GET     /health
GET     /api/users
GET     /api/users/:id
POST    /api/users
PATCH   /api/users/:id
DELETE  /api/users/:id
```

Verbose:

```bash
rustframe routes --verbose
```

```text
GET /api/users/:id
Controller: UsersController
Handler: find_one
Guards:
  - JwtAuthGuard
Interceptors:
  - LoggingInterceptor
Parameters:
  - id: Uuid
```

## Dependency Graph

```bash
rustframe graph
rustframe graph --format text
rustframe graph --format json
rustframe graph --format mermaid
rustframe graph --format dot
```

The graph should show:

- Modules
- Imports
- Providers
- Controllers
- Dependencies
- Exports

It should detect:

- Circular dependencies
- Missing providers
- Private provider access
- Duplicate registrations
- Invalid exports

## Doctor Command

```bash
rustframe doctor
```

Example:

```text
Rust version          OK
Cargo version         OK
Framework version     OK
CLI compatibility     OK
Project configuration OK
Axum adapter          OK
OpenAPI integration   Enabled
Database connection   OK
Environment file      Found
```

## Info Command

```bash
rustframe info
```

Potential output:

- Framework version
- CLI version
- Rust version
- Platform adapter
- Enabled features
- Workspace packages
- Project root

---

# Error Model

Framework errors should be structured.

```rust
pub enum FrameworkError {
    Module(ModuleError),
    Provider(ProviderError),
    Route(RouteError),
    Platform(PlatformError),
    Configuration(ConfigError),
    Lifecycle(LifecycleError),
}
```

Good error messages should include:

- Error code
- Human-readable message
- Related type names
- Resolution path
- Suggested fix

Example:

```text
ProviderResolutionError:
Unable to resolve `UsersService`.

Dependency chain:
UsersController -> UsersService -> UserRepository

Missing provider:
UserRepository

Suggested fix:
Register `UserRepository` in `UsersModule.providers` or import a module that exports it.
```

---

# Security

Security principles:

- No custom cryptography
- Safe defaults
- Secret redaction
- Request size limits
- Header validation
- CORS configuration
- Rate limiting integration
- Security header middleware
- Input validation
- Panic isolation where practical
- Dependency audits

Recommended commands for CI:

```bash
cargo audit
cargo deny check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

---

# Performance

Performance goals should focus on avoiding unnecessary overhead.

Principles:

- Compile-time metadata
- Minimal dynamic dispatch in hot paths
- Cached provider resolution
- Singleton providers by default
- No unnecessary allocations
- Efficient request extensions
- Native Tower middleware support
- Avoid request-scoped DI in early releases

Benchmarks should compare:

- Raw Axum
- Axum with common middleware
- RustFrame over Axum
- Singleton resolution
- Transient resolution
- Route registration time
- Startup time

The framework should be transparent about overhead.

---

# Documentation Strategy

The repository should include:

```text
docs/
├── getting-started.md
├── fundamentals/
│   ├── modules.md
│   ├── dependency-injection.md
│   ├── controllers.md
│   ├── providers.md
│   └── lifecycle.md
├── techniques/
│   ├── validation.md
│   ├── configuration.md
│   ├── logging.md
│   ├── database.md
│   └── caching.md
├── cli/
│   ├── overview.md
│   ├── commands.md
│   └── generators.md
├── testing/
├── recipes/
└── migration-guides/
```

Every feature should include:

- Concept explanation
- Minimal example
- Complete example
- Error cases
- Testing example
- CLI usage
- Generated code explanation

---

# Release Roadmap

## Version 0.1

Core framework:

- Application bootstrap
- Modules
- Singleton providers
- Transient providers
- Constructor injection
- Controllers
- HTTP routing
- Axum adapter
- Middleware
- Guards
- Validation
- Interceptors
- Error handling
- Lifecycle hooks
- Testing module
- CLI project generation
- CLI module, controller, service, and resource generators

## Version 0.2

Developer experience:

- OpenAPI
- Route inspector
- Dependency graph
- Doctor command
- Typed configuration
- Health checks
- Structured logging
- Graceful shutdown
- SQLx example
- JWT example

## Version 0.3

Application features:

- Caching
- Rate limiting
- Scheduling
- Events
- WebSockets
- SSE
- More advanced CLI integrations

## Version 0.4

Distributed systems:

- Microservice abstraction
- NATS
- RabbitMQ
- Kafka
- Redis messaging
- gRPC

## Version 0.5

Advanced architecture:

- CQRS
- Sagas
- GraphQL
- Federation
- Devtools UI
- Plugin ecosystem

---

# Repository Milestones

```text
M0  Architecture and RFCs
M1  Application kernel
M2  Dependency injection
M3  Module compiler
M4  Axum platform adapter
M5  Controllers and routing
M6  Request pipeline
M7  Procedural macros
M8  Testing utilities
M9  CLI foundation
M10 Code generators
M11 Documentation and examples
M12 v0.1.0
```

---

# RFC Process

Use RFCs for major decisions.

```text
rfcs/
├── 0001-module-system.md
├── 0002-dependency-injection.md
├── 0003-controller-routing.md
├── 0004-request-lifecycle.md
├── 0005-provider-scopes.md
├── 0006-platform-adapters.md
└── 0007-cli-generation.md
```

Each RFC should contain:

1. Summary
2. Motivation
3. Goals
4. Non-goals
5. Proposed API
6. Internal architecture
7. Error behavior
8. Alternatives
9. Performance impact
10. Testing strategy
11. Migration considerations
12. Unresolved questions

---

# Contribution Guidelines

## Development Setup

```bash
git clone <repository-url>
cd rustframe
cargo build --workspace
cargo test --workspace
```

## Code Quality

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Pull Requests

A pull request should include:

- Clear problem statement
- Implementation summary
- Tests
- Documentation changes
- Benchmark results for performance-sensitive changes
- RFC reference for architectural changes

## Commit Style

Suggested format:

```text
feat(di): add singleton provider resolution
fix(cli): preserve module formatting during registration
docs(modules): add import and export examples
test(core): cover circular dependency detection
```

---

# Example Application

A complete example should include:

- User registration
- User login
- JWT authentication
- Role guard
- PostgreSQL with SQLx
- Request validation
- Pagination
- Structured errors
- OpenAPI
- Health endpoint
- Integration tests

Structure:

```text
examples/complete-api/
├── Cargo.toml
├── rustframe.toml
├── .env.example
├── migrations/
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── config/
│   ├── common/
│   │   ├── errors.rs
│   │   ├── auth.rs
│   │   └── pagination.rs
│   └── modules/
│       ├── auth/
│       └── users/
└── tests/
```

---

# Initial Public API

```rust
use rustframe::prelude::*;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Injectable)]
pub struct UserRepository {
    pool: PgPool,
}

#[derive(Injectable)]
pub struct UsersService {
    repository: Arc<UserRepository>,
}

impl UsersService {
    pub async fn find_one(
        &self,
        id: Uuid,
    ) -> Result<UserResponse, AppError> {
        todo!()
    }

    pub async fn create(
        &self,
        dto: CreateUserDto,
    ) -> Result<UserResponse, AppError> {
        todo!()
    }
}

#[derive(Injectable)]
#[controller("/users")]
pub struct UsersController {
    service: Arc<UsersService>,
}

#[routes]
impl UsersController {
    #[get("/:id")]
    #[use_guard(JwtAuthGuard)]
    async fn find_one(
        &self,
        #[param] id: Uuid,
    ) -> Result<Json<UserResponse>, AppError> {
        let user = self.service.find_one(id).await?;
        Ok(Json(user))
    }

    #[post("/")]
    #[status(CREATED)]
    async fn create(
        &self,
        #[body] dto: Validated<CreateUserDto>,
    ) -> Result<Json<UserResponse>, AppError> {
        let user = self.service.create(dto.into_inner()).await?;
        Ok(Json(user))
    }
}

#[derive(Module)]
#[module(
    controllers = [UsersController],
    providers = [UsersService, UserRepository],
)]
pub struct UsersModule;

#[derive(Module)]
#[module(
    imports = [UsersModule],
)]
pub struct AppModule;

#[rustframe::main]
async fn main() -> FrameworkResult<()> {
    RustFrame::create::<AppModule>()
        .global_interceptor(TracingInterceptor)
        .global_error_handler(DefaultErrorHandler)
        .listen("0.0.0.0:3000")
        .await
}
```

---

# Implementation Order

The recommended implementation order is critical.

## Step 1: Explicit Core API

Build without macros.

```rust
pub trait Module {
    fn definition() -> ModuleDefinition;
}

pub trait Provider: Send + Sync + 'static {
    fn create(container: &Container) -> Result<Self, ResolveError>
    where
        Self: Sized;
}
```

Success condition:

```rust
let app = FrameworkApplication::builder()
    .module(AppModule::definition())
    .build()
    .await?;
```

## Step 2: Dependency Injection

Implement:

- Registration
- Singleton resolution
- Transient resolution
- Error paths
- Circular dependency detection

## Step 3: Module Compiler

Implement:

- Imports
- Exports
- Controller registration
- Dependency visibility

## Step 4: Axum Adapter

Implement:

- Basic routes
- JSON body
- Path parameters
- Query parameters
- Framework responses

## Step 5: Request Pipeline

Implement:

- Middleware
- Guards
- Validation
- Interceptors
- Error handling

## Step 6: Macros

Add:

```rust
#[derive(Module)]
#[derive(Injectable)]
#[controller]
#[routes]
#[get]
#[post]
#[body]
#[query]
#[param]
```

Macros should only generate calls to already-tested core APIs.

## Step 7: Testing Utilities

Implement:

- Test module
- Provider overrides
- Test application
- Test client

## Step 8: CLI

Implement:

- `new`
- `start`
- `build`
- `test`
- `generate module`
- `generate controller`
- `generate service`
- `generate resource`
- `doctor`

## Step 9: Production Features

Implement:

- Configuration
- OpenAPI
- Health checks
- Logging
- Graceful shutdown
- Database integration examples

---

# Final Direction

The strongest initial positioning is:

> RustFrame is a modular, type-safe application framework for building structured Rust APIs on top of Axum.

The first release should focus on excellence in:

1. Modules
2. Dependency injection
3. Controllers
4. Routing
5. Guards
6. Validation
7. Interceptors
8. Error handling
9. Testing
10. CLI generation

Do not begin with every advanced feature.

A reliable kernel, strong CLI, excellent documentation, and a realistic example application will provide a better foundation than a large collection of unfinished integrations.
