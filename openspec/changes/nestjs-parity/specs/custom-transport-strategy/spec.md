## ADDED Requirements

### Requirement: CustomTransportStrategy trait
The framework SHALL provide a `CustomTransportStrategy` trait allowing users to implement custom transport backends.

#### Scenario: Implement custom transport
- **WHEN** a user implements `CustomTransportStrategy` for their backend
- **THEN** it can be used with the microservice server/client pipeline

### Requirement: Custom strategy registration
The framework SHALL allow custom transport strategies to be registered with the application builder.

#### Scenario: Register custom strategy
- **WHEN** a user calls `app.connect_microservice(CustomStrategy::new(config))`
- **THEN** the custom strategy is managed by the application lifecycle (startup, shutdown)
