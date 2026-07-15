## ADDED Requirements

### Requirement: System SHALL provide HealthIndicator trait
The framework SHALL provide a public `HealthIndicator` trait that components can implement to report their health status.

#### Scenario: Custom health indicator registered
- **WHEN** a component implements `HealthIndicator` with `name()` returning `"my_service"` and `check()` returning `Ok`
- **THEN** it SHALL be discoverable by the health endpoint

### Requirement: Health endpoint SHALL aggregate all registered health checks
The `GET /health` endpoint SHALL discover all registered `HealthIndicator` implementations via the DI container and return their statuses.

#### Scenario: Composite health response
- **WHEN** `GET /health` is called
- **AND** a database health indicator returns `Ok`
- **AND** a Redis health indicator returns `Err`
- **THEN** the response SHALL be `{"status": "degraded", "checks": {"database": "ok", "redis": "unreachable"}}`
- **AND** the HTTP status code SHALL be `200` for `ok`, `207` for `degraded`, `503` for `unhealthy`

### Requirement: Existing IntegrationHealth SHALL be wrapped as HealthIndicator
All existing `IntegrationHealth` implementations (SQLx, SeaORM, Diesel, Mongo, Redis) SHALL provide `HealthIndicator` implementations automatically when their feature is enabled.

#### Scenario: Database health check via IntegrationHealth
- **WHEN** a SQLx pool is registered
- **THEN** it SHALL be available as a HealthIndicator without additional configuration

### Requirement: Health checks SHALL have configurable timeout
Each health check SHALL have a configurable timeout to prevent a stuck check from blocking the endpoint.

#### Scenario: Health check timeout
- **WHEN** a health check takes longer than the configured timeout (default 5s)
- **THEN** that check SHALL be reported as `"unhealthy"` with a timeout message
- **AND** the endpoint SHALL still respond for other checks
