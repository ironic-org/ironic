## ADDED Requirements

### Requirement: Outbound retry for inter-service HTTP calls
The framework SHALL provide retry middleware for outbound HTTP calls made via `HttpClientService`.

#### Scenario: HTTP client retries on failure
- **WHEN** an HTTP request fails with a retryable status (5xx)
- **THEN** the client retries up to the configured `max_retries` with exponential backoff

### Requirement: Outbound circuit breaker for inter-service calls
The framework SHALL provide circuit breaker middleware for outbound HTTP calls made via `HttpClientService`.

#### Scenario: Circuit breaker opens on repeated failures
- **WHEN** requests to a downstream service fail repeatedly
- **THEN** the circuit breaker opens and subsequent requests fast-fail without network calls
- **AND** after the recovery timeout, a probe request is sent to test recovery
