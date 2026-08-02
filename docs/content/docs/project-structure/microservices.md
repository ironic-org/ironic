---
title: Microservice Communication
description: How services communicate — gRPC, events, Redis transport, and HTTP
---

# Microservice Communication

Services in a monorepo communicate through four mechanisms:

| Pattern | Mechanism | When to Use |
|---------|-----------|-------------|
| **gRPC** | Tonic (synchronous) | Request-response between services |
| **Events** | Kafka (asynchronous) | Fire-and-forget background jobs |
| **Redis Transport** | Pub/sub (request-reply) | Lightweight inter-service calls |
| **HTTP Client** | `HttpClientService` | External APIs or REST-only services |

> **Project structure vs framework:** This page explains *how* services in a
> monorepo talk to each other. For the microservice *framework* itself
> (transports, `#[message_handler]`, `#[event_handler]`), see
> [Microservices (Distributed)](/docs/distributed/microservices).

## 1. gRPC Communication

```
┌──────────┐    gRPC call    ┌──────────┐
│ Service A │ ──────────────▶ │ Service B │
│ (Client)  │ ◀────────────── │ (Server)  │
└──────────┘    response     └──────────┘
```

### Shared Proto Library (`libs/proto/`)

```
libs/proto/
├── Cargo.toml
├── build.rs
├── src/lib.rs
└── proto/
    └── greeter.proto
```

**proto/greeter.proto:**
```protobuf
syntax = "proto3";
package greeter;

service Greeter {
    rpc SayHello (HelloRequest) returns (HelloReply);
}

message HelloRequest { string name = 1; }
message HelloReply   { string message = 1; }
```

**build.rs:**
```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/greeter.proto")?;
    Ok(())
}
```

**src/lib.rs:**
```rust
tonic::include_proto!("greeter");
```

### Service B — gRPC Server

```rust
use proto::greeter::{
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use tonic::{Request, Response};

pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, tonic::Status> {
        let name = request.into_inner().name;
        Ok(Response::new(HelloReply {
            message: format!("Hello, {name}!"),
        }))
    }
}

// In main.rs
let addr = "[::1]:50051".parse()?;
tonic::transport::Server::builder()
    .add_service(GreeterServer::new(MyGreeter))
    .serve(addr)
    .await?;
```

### Service A — gRPC Client

```rust
use proto::greeter::greeter_client::GreeterClient;

let mut client = GreeterClient::connect("http://[::1]:50051").await?;
let response = client
    .say_hello(tonic::Request::new(HelloRequest {
        name: "Alice".into(),
    }))
    .await?;
println!("{}", response.into_inner().message);
```

## 2. Kafka Events (Async Background Jobs)

```
┌──────────┐   Kafka Topic   ┌──────────┐
│ Service A │ ──publish──▶   │ Service B │
│ (Producer)│                │ (Consumer)│
└──────────┘                 └──────────┘
```

### Service A — Event Producer

```rust
use ironic::distributed::transport_kafka::{KafkaClient, KafkaClientConfig};

let client = KafkaClient::new(KafkaClientConfig {
    brokers: "localhost:9092".into(),
    topic: "orders".into(),
});
client.connect().await?;

// Fire-and-forget event
client
    .emit("order.created", &OrderEvent {
        id: "order-123".into(),
        amount: 99.99,
    })
    .await?;
```

### Service B — Event Consumer

```rust
use ironic::event_handler;

#[event_handler(transport = "order.created")]
async fn handle_order_created(event: Arc<OrderEvent>) {
    tracing::info!("processing order: {}", event.id);
    // Process background job
}
```

## 3. Redis Transport (Request-Reply)

```
┌──────────┐   Redis Pub/Sub   ┌──────────┐
│ Service A │ ──request─────▶  │ Service B │
│ (Client)  │ ◀──response───   │ (Server)  │
└──────────┘                   └──────────┘
```

### Service B — Message Handler

```rust
use ironic::distributed::transport_redis::{RedisServer, RedisServerConfig};
use std::sync::Arc;

let server = RedisServer::new(RedisServerConfig::default());

server.on_message("user.get", Arc::new(|payload, _ctx| {
    Box::pin(async move {
        let req: GetUserRequest = serde_json::from_slice(&payload)?;
        let user = db.find_user(req.id).await;
        let response = serde_json::to_vec(&user)?;
        Ok(response)
    })
}));

server.listen().await?;
```

### Service A — Request-Reply Client

```rust
let client = RedisClient::new(RedisClientConfig::default());
client.connect().await?;

let user: User = client
    .send("user.get", &GetUserRequest { id: 1 })
    .await?;

println!("got user: {}", user.name);
```

## 4. HTTP Client (Inter-Service REST)

```
┌──────────┐  HTTP Request   ┌──────────┐
│ Service A │ ──────────────▶ │ Service B │
│ (Client)  │ ◀────────────── │ (REST)    │
└──────────┘   Response      └──────────┘
```

```rust
use ironic::services::http_client::HttpClientService;

let http = HttpClientService::new();

// Simple GET
let user: User = http.get("http://auth-service:3001/users/1").await?;

// POST with retry
let created: User = http
    .with_retry(3, 100)
    .post("http://auth-service:3001/users", &CreateUserDto {
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .await?;

// With circuit breaker protection
let user: User = http
    .with_circuit_breaker("auth-service")
    .get("http://auth-service:3001/users/1")
    .await?;
```

## Communication Decision Matrix

| Criteria | gRPC | Kafka | Redis Transport | HTTP |
|----------|------|-------|-----------------|------|
| Synchronous | ✅ | ❌ | ✅ | ✅ |
| Async / Background | ❌ | ✅ | ❌ | ❌ |
| Typed contracts | ✅ (proto) | 🟡 (serde) | 🟡 (serde) | 🟡 (JSON) |
| Performance | ⚡ Fast | 🟡 Medium | ⚡ Fast | 🟡 Medium |
| Service discovery | Manual | Brokered | Brokered | Manual/Registry |
| Best for | Internal APIs | Background jobs | Lightweight RPC | External APIs |
