## ADDED Requirements

### Requirement: Rate limiting SHALL emit standard rate limit headers
The rate limit middleware SHALL emit `X-RateLimit-Limit` and `X-RateLimit-Reset` headers in addition to `X-RateLimit-Remaining`.

#### Scenario: Full set of rate limit headers
- **WHEN** a request passes rate limit middleware
- **THEN** the response SHALL include `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers

## MODIFIED Requirements

### Requirement: Framework SHALL provide built-in rate limiting middleware
The framework SHALL provide configurable rate limiting with in-memory (development) and Redis (production) backends, selected via the `RateLimitBackend` trait.

#### Scenario: Rate limit exceeded returns 429
- **WHEN** a client exceeds the configured rate limit (e.g., 100 requests per minute)
- **THEN** the middleware SHALL return a 429 Too Many Requests response with `Retry-After`, `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers

#### Scenario: Rate limit resets after window
- **WHEN** a client's rate limit window expires
- **THEN** the client SHALL be able to make requests again

#### Scenario: Redis backend enforces global rate limit
- **WHEN** the `RedisRateLimiter` backend is configured
- **AND** a client makes requests across multiple application instances
- **THEN** the rate limit SHALL be enforced globally across all instances
