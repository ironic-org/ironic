## ADDED Requirements

### Requirement: Redis-backed distributed rate limiting
The framework SHALL extend the existing rate limiter to support Redis-backed distributed counters.

#### Scenario: Global rate limit across instances
- **WHEN** two application instances share the same Redis backend
- **THEN** rate limit counters are synchronized across instances via Redis INCR + EXPIRE

#### Scenario: Atomic sliding window
- **WHEN** a request exceeds the rate limit
- **THEN** the middleware returns 429 Too Many Requests with a Retry-After header

### Requirement: Configuration
The framework SHALL provide distributed rate limit configuration with Redis connection details.

#### Scenario: Configure distributed rate limiter
- **WHEN** rate limiting is configured with `distributed: true` and Redis config
- **THEN** in-memory counters are replaced with Redis-backed counters
