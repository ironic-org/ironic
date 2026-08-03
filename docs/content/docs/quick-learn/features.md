---
title: Features
description: Security, observability, database & storage integrations, and the transports (HTTP, gRPC, GraphQL, WebSocket, SSE) that Ironic supports.
---

# Features

This page covers the cross-cutting features and integrations: security, observability,
database & storage, and transports.

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

Requires the `metrics` feature:

```rust
use ironic::metrics::{MetricsLayer, MetricsConfig};

.platform(
    AxumAdapter::new()
        .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
)
```

Metrics are exposed at `GET /metrics`.

### Distributed Tracing (OTLP)

Requires the `telemetry` feature:

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

See the [SSE guide](/docs/transport/sse) for the full walkthrough.
