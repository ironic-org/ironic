## ADDED Requirements

### Requirement: Live Redis transport backend
The framework SHALL provide a live Redis pub/sub implementation of `MicroserviceClient` and `MicroserviceServer`.

#### Scenario: Client publishes message and receives response
- **WHEN** `RedisClient::send("pattern", data)` is called
- **THEN** the client publishes to `pattern` channel, subscribes to `pattern.reply` channel, waits for correlation ID match, and returns the deserialized response

#### Scenario: Server listens and dispatches
- **WHEN** `RedisServer::listen()` is called
- **THEN** the server subscribes to all registered pattern channels and dispatches incoming messages to handlers via pub/sub

#### Scenario: Automatic reconnection
- **WHEN** the Redis connection drops
- **THEN** the client/server automatically reconnects with configurable retry strategy

#### Scenario: Wildcard pattern support
- **WHEN** configured with `wildcards: true`
- **THEN** the server uses Redis `psubscribe`/`pmessage` for wildcard pattern matching

### Requirement: Redis transport configuration
The framework SHALL provide `RedisClientConfig` and `RedisServerConfig` with host, port, retry settings, and serializer/deserializer.

#### Scenario: Custom serialization
- **WHEN** a custom serializer/deserializer is provided in config
- **THEN** the transport uses it instead of the default JSON serializer
