# Operational Endpoints

## Purpose

Standard production endpoints for version info, liveness probes, and readiness probes.

## Requirements

### Requirement: System SHALL provide a version endpoint
A `GET /version` endpoint SHALL return build metadata including git SHA, build timestamp, Rust compiler version, and active feature flags.

#### Scenario: Version endpoint returns build info
- **WHEN** `GET /version` is called
- **THEN** the response SHALL include `git_sha` (string)
- **AND** the response SHALL include `build_timestamp` (RFC 3339 string)
- **AND** the response SHALL include `rust_version` (string, e.g. "1.97.0")
- **AND** the response SHALL include `features` (array of active feature flag strings)
- **AND** the response SHALL include `version` (semver string, e.g. "0.4.8")

#### Scenario: Version endpoint is accessible
- **WHEN** `GET /version` is called
- **THEN** the HTTP status SHALL be `200 OK`
- **AND** the response SHALL be `Content-Type: application/json`

### Requirement: System SHALL provide a liveness probe endpoint
A `GET /health/live` endpoint SHALL return `200 OK` with `{"status": "alive"}` if the process is running, regardless of dependency health.

#### Scenario: Liveness probe responds
- **WHEN** `GET /health/live` is called
- **THEN** the HTTP status SHALL be `200 OK`
- **AND** the response SHALL be `{"status": "alive"}`

### Requirement: System SHALL provide a readiness probe endpoint
A `GET /health/ready` endpoint SHALL aggregate all registered `HealthIndicator` implementations and return the composite result.

#### Scenario: All dependencies healthy
- **WHEN** `GET /health/ready` is called
- **AND** all registered health indicators return `Ok`
- **THEN** the HTTP status SHALL be `200 OK`
- **AND** the response SHALL be `{"status": "ok", "checks": {...}}`

#### Scenario: Dependency degraded
- **WHEN** `GET /health/ready` is called
- **AND** one or more health indicators return `Err`
- **THEN** the HTTP status SHALL be `503 Service Unavailable`
- **AND** the response SHALL include the failing check details

### Requirement: Build info SHALL be embedded at compile time
The `build.rs` script SHALL capture git SHA, build timestamp, and Rust compiler version from environment variables or fallback values.

#### Scenario: Build with CI env vars
- **WHEN** the project is built in CI with `GIT_SHA` and `BUILD_TIMESTAMP` set
- **THEN** the binary SHALL contain these values in the version response

#### Scenario: Build without CI env vars
- **WHEN** the project is built locally without CI env vars
- **THEN** the binary SHALL report `"unknown"` for git SHA and the current time for build timestamp
