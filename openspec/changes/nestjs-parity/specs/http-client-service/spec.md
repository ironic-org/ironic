## ADDED Requirements

### Requirement: HttpClientService
The framework SHALL provide an injectable `HttpClientService` for making HTTP requests to other services.

#### Scenario: GET request with typed response
- **WHEN** `http_client.get::<User>("http://users/1").await`
- **THEN** it sends a GET request, deserializes the JSON response to `User`, and returns it

#### Scenario: POST request with body
- **WHEN** `http_client.post::<CreateUser, User>("http://users", body).await`
- **THEN** it sends a POST with JSON body, deserializes response, and returns it

### Requirement: Outbound resilience
The `HttpClientService` SHALL support retry and circuit breaker middleware.

#### Scenario: Retry on failure
- **WHEN** a request fails with a retryable status
- **THEN** it retries up to the configured number of times with exponential backoff

#### Scenario: Circuit breaker integration
- **WHEN** a downstream service is failing repeatedly
- **THEN** the circuit breaker opens and requests fast-fail without hitting the failing service

### Requirement: Service discovery integration
The `HttpClientService` SHALL resolve service names through the service discovery abstraction.

#### Scenario: Resolve by service name
- **WHEN** `http_client.get::<User>("service://users-api/profile").await`
- **THEN** it resolves "users-api" through the service registry and replaces with the actual address
