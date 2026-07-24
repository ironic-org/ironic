## ADDED Requirements

### Requirement: Live RabbitMQ transport backend
The framework SHALL provide a live RabbitMQ implementation of `MicroserviceClient` and `MicroserviceServer` using the `lapin` crate.

#### Scenario: Client publishes and receives
- **WHEN** `RmqClient::send("pattern", data)` is called
- **THEN** the client publishes to the configured exchange with routing key = pattern, and listens for replies on the reply-to queue

#### Scenario: Server consumes from queue
- **WHEN** `RmqServer::listen()` is called
- **THEN** the server asserts the configured queue, binds to the exchange, and consumes messages dispatching registered handlers

#### Scenario: Exchange type configuration
- **WHEN** configured with `exchange_type: "topic"`
- **THEN** the server binds using topic exchange routing with wildcard support

### Requirement: RabbitMQ transport configuration
The framework SHALL provide `RmqClientConfig` and `RmqServerConfig` with URL, exchange, queue, prefetch count, and serializer/deserializer.

#### Scenario: Durable queue
- **WHEN** configured with `queue_options: durable`
- **THEN** the server asserts a durable queue that survives broker restarts
