## ADDED Requirements

### Requirement: W3C trace context propagation
The framework SHALL automatically inject and extract W3C `traceparent` and `tracestate` headers across transport and HTTP boundaries.

#### Scenario: Trace context injected into transport envelope
- **WHEN** a microservice client sends a message via transport
- **THEN** the current trace context is automatically added to the envelope headers

#### Scenario: Trace context extracted on server
- **WHEN** a microservice server receives a message with trace headers
- **THEN** a child span is created from the incoming trace context, linking spans across services

#### Scenario: Trace context in HTTP client
- **WHEN** `HttpClientService` makes an outbound HTTP call
- **THEN** `traceparent` and `tracestate` headers are automatically injected into the request
