# Observability

## Purpose

Tracing integration, request IDs, structured logging, health endpoints, and security defaults.

## MODIFIED Requirements

### Requirement: Framework SHALL provide a health endpoint
The framework SHALL support registering health check endpoints that report the status of application dependencies, aggregated from all registered `HealthIndicator` implementations. The system SHALL expose liveness (`/health/live`) and readiness (`/health/ready`) probe endpoints in addition to the existing composite `/health` endpoint.

#### Scenario: Liveness endpoint returns alive
- **WHEN** a GET request is made to `/health/live`
- **THEN** the response SHALL be `{"status": "alive"}` with HTTP 200

#### Scenario: Readiness endpoint reflects dependency failure
- **WHEN** a GET request is made to `/health/ready`
- **AND** a database health check fails
- **THEN** the response SHALL include `{"checks": {"database": "unreachable"}}`
- **AND** the aggregate status SHALL be `"degraded"` or `"unhealthy"`
- **AND** the HTTP status SHALL be 503

#### Scenario: Composite health endpoint returns aggregate status (unchanged)
- **WHEN** a GET request is made to `/health`
- **THEN** the response SHALL include the aggregate application health status with per-dependency details

#### Scenario: Health endpoint reflects dependency failure (unchanged)
- **WHEN** a database health check fails
- **THEN** the response SHALL include `{"checks": {"database": "unreachable"}}`
- **AND** the aggregate status SHALL be `"degraded"` or `"unhealthy"`

#### Scenario: Health endpoint timeout (unchanged)
- **WHEN** a health check exceeds the configured timeout
- **THEN** that check SHALL be reported as `"unhealthy"`
- **AND** other checks SHALL still complete

## ADDED Requirements

### Requirement: System SHALL provide a version endpoint
The framework SHALL expose a `GET /version` endpoint returning build metadata.

#### Scenario: Version response
- **WHEN** `GET /version` is called
- **THEN** the response SHALL include `git_sha`, `build_timestamp`, `rust_version`, `features`, and `version` fields
