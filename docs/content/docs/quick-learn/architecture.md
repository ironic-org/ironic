---
title: Architecture
description: How the framework fits together — application lifecycle, dependency injection, module system, routing, and middleware.
---

# Architecture

This page explains how the Ironic framework fits together: the application lifecycle,
dependency injection, the module system, routing, and the middleware pipeline.

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

## Container Internals

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
