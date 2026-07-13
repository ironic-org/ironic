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
The framework SHALL provide configurable rate limiting with in-memory (development) and Redis (production) backends.

#### Scenario: Rate limit exceeded returns 429
- **WHEN** a client exceeds the configured rate limit (e.g., 100 requests per minute)
- **THEN** the middleware SHALL return a 429 Too Many Requests response with a `Retry-After` header

#### Scenario: Rate limit resets after window
- **WHEN** a client's rate limit window expires
- **THEN** the client SHALL be able to make requests again

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
