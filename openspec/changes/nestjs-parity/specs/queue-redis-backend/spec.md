## ADDED Requirements

### Requirement: RedisQueue documentation
The framework SHALL provide full documentation for `RedisQueue` covering `QueueConfig` fields (name, prefix, visibility_timeout, max_retries), retry count tracking, dead-letter queue, and TTL expiry.

#### Scenario: QueueConfig documentation covers all fields
- **WHEN** a user reads the queues documentation
- **THEN** they can find documentation for `QueueConfig` with all fields and their effects

### Requirement: #[cache_key] and #[cache_ttl] documentation
The framework SHALL provide documentation for the `#[cache_key]` and `#[cache_ttl]` parameter-level decorators in the cache documentation section.

#### Scenario: Cache parameter decorators documented
- **WHEN** a user reads the cache documentation
- **THEN** they can find usage examples for `#[cache_key]` and `#[cache_ttl]`
