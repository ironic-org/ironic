## ADDED Requirements

### Requirement: Distributed rate limiting SHALL support configurable backend
The existing rate limit middleware SHALL support a configurable backend trait, with `InMemoryRateLimiter` (default) and `RedisRateLimiter` implementations.

#### Scenario: Redis backend enforces global rate limit
- **WHEN** the `RedisRateLimiter` backend is configured
- **AND** a client makes requests across multiple application instances
- **THEN** the rate limit SHALL be enforced globally across all instances

#### Scenario: Atomic sliding window with Lua script
- **WHEN** a rate limit decision is made with Redis backend
- **THEN** a Lua script using INCR + EXPIRE SHALL atomically increment and expire the counter

### Requirement: Rate limit backend selection via config
The rate limit middleware SHALL accept a backend parameter to switch between in-memory and Redis.

#### Scenario: Configure Redis backend
- **WHEN** rate limiting is configured with `backend: RateLimitBackend::Redis { url: "...", prefix: "ratelimit" }`
- **THEN** the middleware uses Redis for counter storage
