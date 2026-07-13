## ADDED Requirements

### Requirement: Framework SHALL provide a Redis transport adapter
The framework SHALL implement the existing `Transport` trait over Redis pub/sub channels.

#### Scenario: Redis transport sends and receives messages
- **WHEN** a `RedisTransport` is configured with a Redis URL and channel name
- **AND** a message is sent via `transport.send(msg)`
- **THEN** the message SHALL be published to the Redis channel and received by subscribers

### Requirement: Framework SHALL provide a RabbitMQ transport adapter
The framework SHALL implement the existing `Transport` trait over RabbitMQ queues and exchanges.

#### Scenario: RabbitMQ transport sends and receives messages
- **WHEN** a `RabbitMqTransport` is configured with connection parameters and queue name
- **AND** a message is sent via `transport.send(msg)`
- **THEN** the message SHALL be published to the RabbitMQ queue and consumed by subscribers

### Requirement: Framework SHALL provide a Kafka transport adapter
The framework SHALL implement the existing `Transport` trait over Kafka topics.

#### Scenario: Kafka transport sends and receives messages
- **WHEN** a `KafkaTransport` is configured with broker list and topic name
- **AND** a message is sent via `transport.send(msg)`
- **THEN** the message SHALL be produced to the Kafka topic and consumed by subscribers

### Requirement: Transport adapters SHALL be feature-flagged
Each transport adapter SHALL be gated behind its own feature flag (`transport-redis`, `transport-rabbitmq`, `transport-kafka`).

#### Scenario: Transport feature flag enables adapter
- **WHEN** `transport-redis` feature is enabled in Cargo.toml
- **THEN** the `RedisTransport` struct SHALL be available for use

### Requirement: Transport adapters SHALL support connection configuration
Each transport adapter SHALL accept configuration via a typed builder or config struct.

#### Scenario: Transport is configured via builder
- **WHEN** `RedisTransport::builder().url("redis://...").channel("events").build()`
- **THEN** a `RedisTransport` instance SHALL be returned with the specified configuration
