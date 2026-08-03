---
title: Message Handler
description: Declare request-response message handlers for microservice transport
---

# `#[message]`

The `#[message]` attribute registers an async function as a request-response
handler on a [`MicroserviceServer`].

```rust
#[message("user.get")]
async fn get_user(request: GetUserRequest) -> GetUserResponse {
    // ...
}
```

## How It Works

The macro generates a registration function that:
1. Parses the pattern from the attribute argument
2. Creates a [`MessageHandler`] closure that deserializes the request, calls
   the function, and serializes the response
3. Registers it on the server via [`MicroserviceServer::on_message`]

## Usage

```rust
use ironic::{message, distributed::MicroserviceServer};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct GetUser { id: u64 }

#[derive(Serialize)]
struct User { id: u64, name: String }

#[message("user.get")]
async fn get_user(req: GetUser) -> User {
    User { id: req.id, name: "Alice".into() }
}
```

Register the handler on a server:

```rust
let server = RedisServer::new(config);
__message_reg_get_user(&server);
server.listen().await?;
```

## Request-Response Flow

```
Client                    Server
  │                         │
  ├── send("user.get", ────▶│
  │    data)                │
  │                         ├── deserialize request
  │                         ├── call handler
  │                         ├── serialize response
  │◀─── response ──────────┤
  │                         │
```

## Transport Integration

Message handlers work with any transport that implements `MicroserviceServer`:
Redis, RabbitMQ, Kafka, TCP, or custom backends.

## See Also

- [`#[event]`](../transport/events) — fire-and-forget event handlers
- [Microservices Overview](./microservices) — architecture and setup
- [Transport Configuration](./message-transports) — backend settings
