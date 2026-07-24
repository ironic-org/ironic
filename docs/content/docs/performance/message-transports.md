---
title: Transport Backends
description: Configure Redis, RabbitMQ, Kafka, and TCP microservice transports
---

# Transport Backends

## Redis

```toml
[dependencies]
ironic = { features = ["transport-redis"] }
```

```rust
use ironic::distributed::transport_redis::{
    RedisClient, RedisClientConfig,
    RedisServer, RedisServerConfig,
};

// Server
let server = RedisServer::new(RedisServerConfig {
    url: "redis://127.0.0.1:6379".into(),
    wildcards: false,
    retry_attempts: 3,
    retry_delay_ms: 1000,
    ..Default::default()
});

// Client
let client = RedisClient::new(RedisClientConfig {
    url: "redis://127.0.0.1:6379".into(),
    retry_attempts: 3,
    retry_delay_ms: 1000,
    ..Default::default()
});
```

Uses Redis pub/sub with `PUBLISH`/`SUBSCRIBE`. Replies are sent on `{pattern}.reply` channels.

## RabbitMQ

```toml
[dependencies]
ironic = { features = ["transport-rabbitmq"] }
```

```rust
use ironic::distributed::transport_rabbitmq::{
    RmqClient, RmqClientConfig,
    RmqServer, RmqServerConfig,
};

let server = RmqServer::new(RmqServerConfig {
    url: "amqp://guest:guest@127.0.0.1:5672".into(),
    exchange: "ironic".into(),
    queue: String::new(),
    ..Default::default()
});
```

Uses topic exchanges with queue binding. Replies use AMQP's `reply-to` mechanism.

## Kafka

```toml
[dependencies]
ironic = { features = ["transport-kafka"] }
```

```rust
use ironic::distributed::transport_kafka::{
    KafkaClient, KafkaClientConfig,
    KafkaServer, KafkaServerConfig,
};
```

Uses topics with producer/consumer pattern. Reply topic is `{topic}_reply`.

## TCP

```rust
use ironic::distributed::transport_tcp::{
    TcpClient, TcpClientConfig,
    TcpServer, TcpServerConfig,
};
```

Simple TCP socket transport. Messages are newline-delimited JSON.

## In-Memory

For testing, use `InMemoryServer::pair()`:

```rust
use ironic::distributed::microservices::InMemoryServer;

let (client, server) = InMemoryServer::pair(16);
```
