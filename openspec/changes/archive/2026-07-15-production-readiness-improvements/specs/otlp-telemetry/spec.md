## ADDED Requirements

### Requirement: System SHALL export traces via OTLP gRPC when configured
The framework SHALL send OpenTelemetry spans to a configured OTLP collector (Jaeger, Tempo, Datadog) via gRPC when `TelemetryConfig.otlp_endpoint` is set and the `telemetry` feature is enabled.

#### Scenario: Traces exported to OTLP collector
- **WHEN** `TelemetryConfig` has `otlp_endpoint = Some("http://localhost:4317")` and the `telemetry` feature is enabled
- **THEN** tracing spans SHALL be exported to the configured OTLP endpoint

#### Scenario: No OTLP export when endpoint is None
- **WHEN** `TelemetryConfig` has `otlp_endpoint = None`
- **THEN** tracing SHALL work locally (stdout) without attempting OTLP export

### Requirement: System SHALL propagate W3C trace context headers
The framework SHALL inject `traceparent` and `tracestate` headers into outgoing HTTP requests when `propagate_context` is true.

#### Scenario: Trace context propagated in outgoing request
- **WHEN** an outgoing HTTP request is made within a tracing span
- **THEN** the request SHALL include W3C `traceparent` header with the current trace ID, span ID, and trace flags

### Requirement: System SHALL support configurable trace sampling
The framework SHALL respect `TelemetryConfig.sample_rate` for probabilistic sampling decisions.

#### Scenario: Sampling rate filters spans
- **WHEN** `sample_rate` is set to `0.25`
- **THEN** approximately 25% of requests SHALL produce exported spans
- **AND** all requests SHALL still get local tracing spans (sampling only affects export)

### Requirement: System SHALL emit semantic convention span attributes
The request tracing middleware SHALL set Semantic Conventions attributes (`http.method`, `http.url`, `http.status_code`, `http.request_id`) on the tracing span.

#### Scenario: Span has semantic attributes
- **WHEN** a request completes
- **THEN** the tracing span SHALL have attributes `http.method`, `http.url`, `http.status_code` per OpenTelemetry semantic conventions
