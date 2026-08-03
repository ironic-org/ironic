---
title: Patterns
description: Common application patterns — repository/service/controller layers, caching, scheduling, events, CQRS, sagas, configuration, best practices, and troubleshooting.
---

# Patterns

This page collects common application patterns, configuration approaches, best
practices, and troubleshooting guidance.

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

#[event]
async fn on_user_created(event: Arc<UserCreatedEvent>, bus: Arc<EventBus>) {
    println!("User created: {}", event.email);
    // The event handler is auto-registered by the `#[event]` macro
}

// Publishing:
bus.publish(UserCreatedEvent { user_id: 1, email: "test@example.com" }).await;
```

### Message Handlers (Microservices)

Requires the `microservices` feature:

```rust
use ironic::prelude::*;

#[message("user.created")]
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

See the [SSE guide](/docs/transport/sse) for the full walkthrough.

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
#[message("command.create_user")]
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
#[message("query.get_user")]
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

---

## Testing

The framework supports both unit tests (constructing services directly) and
integration tests (full HTTP via `TestApplication`). See the dedicated
[Testing section](/docs/testing/testing) for a complete guide, plus pages on
[test application setup](/docs/testing/test-application), [mocking](/docs/testing/mocking),
and [CI](/docs/testing/ci).

Quick integration test example:

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

Override providers in tests:

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
