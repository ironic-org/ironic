---
title: Queues and distributed architecture
description: Queues, microservice transports, gRPC, CQRS, sagas, and GraphQL feature modules.
---

# Queues and distributed architecture

The `distributed` feature enables all APIs in this section; each can also be selected separately.

- `queues`: the `Queue` contract and bounded `InMemoryQueue` with acknowledgement/requeue APIs.
- `microservices`: transport-neutral envelopes and connected in-memory duplex endpoints.
- `grpc`: the upstream Tonic API plus DI registration for reusable channels.
- `cqrs`: a typed command/query dispatcher that validates duplicate and missing handlers.
- `sagas`: ordered forward steps with reverse compensation after failure.
- `graphql`: the upstream async-graphql API and schema DI registration.

## Microservice transports

The `Transport` trait defines a bidirectional message endpoint. `ChannelTransport` provides
connected in-memory duplex pairs for development and testing.

External transport adapters are available behind feature flags:

| Feature flag | Adapter | Protocol |
|-------------|---------|----------|
| `transport-redis` | `RedisTransport` | Redis pub/sub |
| `transport-rabbitmq` | `RabbitMqTransport` | AMQP |
| `transport-kafka` | `KafkaTransport` | Kafka topics |

Each adapter has a typed builder for configuration:

```rust
use ironic::distributed::microservices::RedisTransportConfig;

let config = RedisTransportConfig::builder("redis://localhost:6379", "orders-channel")
    .pool_size(8);
let transport = config.connect().await?;
```

```rust
use ironic::distributed::microservices::KafkaTransportConfig;

let config = KafkaTransportConfig::builder("localhost:9092", "events-topic")
    .group_id("processor-1");
let transport = config.connect().await?;
```

Use `ChannelTransport::pair()` for deterministic integration tests:

```rust
let (client, server) = ChannelTransport::pair(16);
client.send(envelope).await?;
let received = server.receive().await?.unwrap();
```

The in-memory queue and channel transport are deterministic development/test implementations. Use a
durable broker adapter for production delivery guarantees. Application message IDs, idempotency,
retry limits, dead-letter handling, tracing propagation, and schema evolution remain explicit
deployment decisions rather than hidden defaults.
