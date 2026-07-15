# Observability

## Purpose

Tracing integration, request IDs, structured logging, health endpoints, and security defaults.

## Requirements

### Requirement: Framework SHALL integrate with tracing crate for structured logging
The framework SHALL use the `tracing` crate as its logging and instrumentation layer, providing spans for request processing lifecycle.

#### Scenario: Request tracing span is created
- **WHEN** a request enters the pipeline
- **THEN** a tracing span SHALL be created with request method, path, and request ID

### Requirement: Framework SHALL support request IDs
The framework SHALL generate or propagate a unique request ID for each incoming request, accessible via request extensions.

#### Scenario: Request ID is generated
- **WHEN** a request arrives without a request ID header
- **THEN** a unique request ID SHALL be generated and attached to the request

### Requirement: Framework SHALL provide a health endpoint
The framework SHALL support registering health check endpoints that report the status of application dependencies, aggregated from all registered `HealthIndicator` implementations.

#### Scenario: Composite health endpoint returns aggregate status
- **WHEN** a GET request is made to `/health`
- **THEN** the response SHALL include the aggregate application health status with per-dependency details

#### Scenario: Health endpoint reflects dependency failure
- **WHEN** a database health check fails
- **THEN** the response SHALL include `{"checks": {"database": "unreachable"}}`
- **AND** the aggregate status SHALL be `"degraded"` or `"unhealthy"`

#### Scenario: Health endpoint timeout
- **WHEN** a health check exceeds the configured timeout
- **THEN** that check SHALL be reported as `"unhealthy"`
- **AND** other checks SHALL still complete

### Requirement: Framework SHALL enforce safe defaults
The framework SHALL provide safe default configuration for request size limits, timeouts, and security headers.

#### Scenario: Request size limit is enforced
- **WHEN** a request body exceeds the configured size limit
- **THEN** the request SHALL be rejected with a payload too large error

### Requirement: System SHALL support OTLP trace export
The framework SHALL export spans to an OpenTelemetry-compatible collector via OTLP gRPC when configured.

#### Scenario: OTLP export active
- **WHEN** `TelemetryConfig.otlp_endpoint` is set
- **AND** the `telemetry` feature is enabled
- **THEN** spans SHALL be exported to the collector

### Requirement: System SHALL propagate W3C trace context
Outgoing HTTP requests SHALL include `traceparent` headers for distributed tracing.

#### Scenario: Trace context propagated
- **WHEN** a traced request makes an outbound HTTP call
- **THEN** the outbound request SHALL include a `traceparent` header

### Requirement: System SHALL emit semantic convention attributes
Tracing spans SHALL include `http.method`, `http.url`, `http.status_code` attributes per OpenTelemetry semantic conventions.

#### Scenario: Span has semantic attributes
- **WHEN** a request completes
- **THEN** the span SHALL have `http.method`, `http.url`, and `http.status_code` attributes

### Requirement: System SHALL provide structured logging API
The framework SHALL provide a convenience API for emitting structured log events with typed fields.

#### Scenario: Structured log event
- **WHEN** application code calls `info!(event = "user.login", user_id = 42)`
- **THEN** the log output SHALL include `event=user.login` and `user_id=42` as structured fields
