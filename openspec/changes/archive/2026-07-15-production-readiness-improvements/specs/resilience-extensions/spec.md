## ADDED Requirements

### Requirement: Graceful shutdown SHALL support configurable drain timeout
The `AxumAdapter` SHALL accept a `drain_timeout` configuration. When the shutdown signal is received, in-flight requests SHALL be given up to the drain timeout to complete before the process exits.

#### Scenario: Requests drain before shutdown
- **WHEN** a shutdown signal is received
- **AND** there are in-flight requests
- **THEN** the server SHALL wait up to the configured `drain_timeout` (default 30s) for them to complete
- **AND** remaining in-flight requests after timeout SHALL be dropped

### Requirement: Rate limit middleware SHALL emit standard headers
The rate limit middleware SHALL emit `X-RateLimit-Limit` (max requests per window) and `X-RateLimit-Reset` (Unix timestamp of window reset) headers in addition to `X-RateLimit-Remaining`.

#### Scenario: All rate limit headers present
- **WHEN** a request passes rate limit middleware
- **THEN** the response SHALL include `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers

### Requirement: System SHALL support backpressure / bulkhead pattern (P3)
The framework SHALL provide a concurrency limiter middleware that rejects requests when a configurable number of in-flight requests is exceeded.

#### Scenario: Concurrency limit exceeded
- **WHEN** the number of in-flight requests exceeds the configured limit (default 256)
- **THEN** new requests SHALL be rejected with a 503 Service Unavailable response
