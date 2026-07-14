---
title: Message Transports
description: Connect to Redis, RabbitMQ, and Kafka for distributed messaging — queues, pub/sub, and microservice communication.
---

# Message Transports

## What you'll learn

- Configure Redis as a message transport
- Connect to RabbitMQ for AMQP messaging
- Set up Kafka for high-throughput event streaming

Enable individually:

```toml
ironic = { features = ["transport-redis"] }
ironic = { features = ["transport-rabbitmq"] }
ironic = { features = ["transport-kafka"] }
```

---

## Redis transport

```rust
use ironic::distributed::microservices::RedisTransportConfig;

let config = RedisTransportConfig {
    url: "redis://localhost:6379".into(),
    channel_prefix: Some("myapp".into()),  // Namespace channels
    pool_size: Some(8),                     // Connection pool
};

let transport = config.connect().await?;
```

### Redis pub/sub

```rust
// Publisher
transport.publish("orders.new", order_payload).await?;

// Subscriber
let mut stream = transport.subscribe("orders.new").await?;
while let Some(msg) = stream.next().await {
    process_order(msg).await;
}
```

## RabbitMQ transport

```rust
use ironic::distributed::microservices::RabbitMqTransportConfig;

let config = RabbitMqTransportConfig {
    url: "amqp://guest:guest@localhost:5672".into(),
    exchange: "myapp.events".into(),
    queue_prefix: Some("myapp".into()),
};

let transport = config.connect().await?;
```

### RabbitMQ routing

```rust
// Publish to exchange with routing key
transport.publish("order.created", &routing_key, payload).await?;

// Bind queue to exchange
transport.bind_queue("orders", "order.*").await?;

// Consume
let mut consumer = transport.consume("orders").await?;
```

## Kafka transport

```rust
use ironic::distributed::microservices::KafkaTransportConfig;

let config = KafkaTransportConfig {
    brokers: "localhost:9092".into(),
    group_id: "myapp-consumers".into(),
    client_id: Some("myapp-producer".into()),
};

let transport = config.connect().await?;
```

### Kafka producer/consumer

```rust
// Producer
transport.produce("orders", key, payload).await?;

// Consumer
let mut stream = transport.consume(&["orders"]).await?;
while let Some(record) = stream.next().await {
    handle_order(record.key, record.value).await;
}
```

## Which transport should I use?

| Transport | Best for | Throughput |
|-----------|----------|-----------|
| **Redis** | Lightweight pub/sub, caching, simple queues | ~100k msg/s |
| **RabbitMQ** | Reliable delivery, complex routing, enterprise | ~50k msg/s |
| **Kafka** | High-volume event streaming, replay, log-based | ~1M+ msg/s |

> **Start with Redis** for simple pub/sub. Move to Kafka when you need event replay or millions of messages per second.

## What you learned

- [x] Redis: simple pub/sub, good for most use cases
- [x] RabbitMQ: reliable delivery with routing keys
- [x] Kafka: high-throughput event streaming
- [x] All transports integrate with Ironic's DI and queues
