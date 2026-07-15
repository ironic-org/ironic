## ADDED Requirements

### Requirement: Framework SHALL provide built-in CORS middleware
The framework SHALL provide configurable CORS middleware supporting origin, method, header, credential, and preflight configuration.

#### Scenario: CORS middleware allows configured origin
- **WHEN** CORS middleware is configured with `allowed_origins = ["https://example.com"]`
- **AND** a request arrives with `Origin: https://example.com`
- **THEN** the response SHALL include `Access-Control-Allow-Origin: https://example.com`

#### Scenario: CORS middleware blocks unlisted origin
- **WHEN** CORS middleware is configured with `allowed_origins = ["https://example.com"]`
- **AND** a request arrives with `Origin: https://evil.com`
- **THEN** the response SHALL NOT include `Access-Control-Allow-Origin`

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

### Requirement: Framework SHALL provide security headers middleware
The framework SHALL provide middleware to set HSTS, CSP, X-Content-Type-Options, and X-Frame-Options headers.

#### Scenario: Security headers are applied
- **WHEN** security headers middleware is configured with HSTS and CSP policies
- **THEN** all responses SHALL include the configured security headers

### Requirement: Framework SHALL provide CSRF protection middleware
The framework SHALL provide CSRF protection middleware using synchronizer token pattern with configurable token generation and validation.

#### Scenario: CSRF token mismatch rejects request
- **WHEN** a state-changing request arrives without a valid CSRF token
- **THEN** the middleware SHALL return a 403 Forbidden response

### Requirement: Rate limiting SHALL emit standard rate limit headers
The rate limit middleware SHALL emit `X-RateLimit-Limit` and `X-RateLimit-Reset` headers in addition to `X-RateLimit-Remaining`.

#### Scenario: Full set of rate limit headers
- **WHEN** a request passes rate limit middleware
- **THEN** the response SHALL include `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers
