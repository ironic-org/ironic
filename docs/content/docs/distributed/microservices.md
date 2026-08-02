---
title: Microservices
description: Build microservice applications with Ironic's transport framework
---

# Microservices

Ironic provides a first-class microservice architecture inspired by NestJS.
Services communicate through transport backends using either **request-response**
(`#[message_handler]`) or **event-based** (`#[event_handler]`) patterns.

## Transport Architecture

```
┌──────────────────┐     ┌──────────────────┐
│   Service A      │     │   Service B      │
│                  │     │                  │
│  MicroserviceClient ───▶ MicroserviceServer │
│  (sends requests │     │  (handles +      │
│   and events)    │     │   replies)       │
└──────────────────┘     └──────────────────┘
```

## Supported Transports

| Transport | Feature Flag | Crate | Description |
|-----------|-------------|-------|-------------|
| Redis | `transport-redis` | `redis` | Pub/sub channels with reply patterns |
| RabbitMQ | `transport-rabbitmq` | `lapin` | Topic exchanges with queue binding |
| Kafka | `transport-kafka` | `kafka` | Topics with consumer groups |
| TCP | `microservices` | `tokio` | Direct TCP socket communication |
| In-Memory | `microservices` | — | Paired channels for testing |

## Getting Started

Enable the transport feature in `Cargo.toml`:

```toml
[dependencies]
ironic = { features = ["transport-redis"] }
```

Create a microservice server:

```rust
use ironic::distributed::transport_redis::{RedisServer, RedisServerConfig};

let server = RedisServer::new(RedisServerConfig {
    url: "redis://127.0.0.1:6379".into(),
    ..Default::default()
});
server.listen().await?;
```

Create a client and send a request:

```rust
use ironic::distributed::transport_redis::{RedisClient, RedisClientConfig};

let client = RedisClient::new(RedisClientConfig::default());
client.connect().await?;
let response: String = client.send("user.get", &GetUser { id: 1 }).await?;
```

## Hybrid Applications

Run HTTP and microservice in the same process:

```rust
Application::builder()
    .module(AppModule)
    .microservice_server(RedisServer::new(config))
    .microservice_client(RedisClient::new(config))
    .platform(AxumAdapter::new())
    .build()
    .await?
    .listen("0.0.0.0:3000")
    .await?;
```

## Pattern Reference

- [`#[message_handler("pattern")]`](./message-handler) — request-response handler
- [`#[event_handler("pattern")]`](../transport/events) — event handler (fire-and-forget)
- [Transport Configuration](./message-transports) — backend-specific settings
- [Distributed Tracing](../observability/distributed-tracing) — W3C trace context propagation
