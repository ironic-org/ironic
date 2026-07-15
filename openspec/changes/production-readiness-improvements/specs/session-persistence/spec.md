## ADDED Requirements

### Requirement: Session store SHALL support Redis backend
The framework SHALL provide a `RedisSessionStore` implementation of the `SessionStore` trait that persists sessions in Redis.

#### Scenario: Session stored in Redis
- **WHEN** a session is created with `RedisSessionStore`
- **THEN** the session data SHALL be written to Redis with a configurable TTL
- **AND** the session SHALL survive application restarts

#### Scenario: Session retrieved from Redis
- **WHEN** a request includes a session cookie
- **AND** the session exists in Redis
- **THEN** the session data SHALL be retrieved and hydrated

### Requirement: Redis sessions SHALL support configurable serialization
The `RedisSessionStore` SHALL support configurable serialization (JSON default, MessagePack optional) for session value encoding.

#### Scenario: JSON serialized session
- **WHEN** a session value is stored
- **THEN** it SHALL be serialized as JSON by default

### Requirement: Session TTL SHALL be configurable
The `RedisSessionStore` SHALL support configurable TTL per session.

#### Scenario: Session expires after TTL
- **WHEN** a session is created with TTL of 3600 seconds
- **THEN** the Redis key SHALL expire after 3600 seconds
