## ADDED Requirements

### Requirement: Live Kafka transport backend
The framework SHALL provide a live Kafka implementation of `MicroserviceClient` and `MicroserviceServer`.

#### Scenario: Client produces message
- **WHEN** `KafkaClient::send("pattern", data)` is called
- **THEN** the client produces a message to the configured topic with key = pattern and the serialized payload, and awaits the response from the reply topic

#### Scenario: Server consumes from topic
- **WHEN** `KafkaServer::listen()` is called
- **THEN** the server subscribes to the configured topic with consumer group, partitions messages by pattern key, and dispatches to registered handlers

### Requirement: Kafka transport configuration
The framework SHALL provide `KafkaClientConfig` and `KafkaServerConfig` with brokers, topic, consumer group, and serializer/deserializer.

#### Scenario: Consumer group management
- **WHEN** multiple server instances share the same consumer group
- **THEN** messages are load-balanced across instances

#### Scenario: Producer-only mode
- **WHEN** configured with `producer_only_mode: true`
- **THEN** the client only produces messages without consuming replies (fire-and-forget only)
