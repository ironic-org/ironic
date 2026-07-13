## ADDED Requirements

### Requirement: Container SHALL support singleton provider registration and resolution
The DI container SHALL support registering a provider type as singleton, where the provider is constructed once and the same instance is returned for all subsequent resolutions.

#### Scenario: Singleton returns same instance
- **WHEN** a provider is registered as singleton and resolved multiple times
- **THEN** the SAME instance SHALL be returned for each resolution

#### Scenario: Singleton initialization is thread-safe
- **WHEN** multiple threads simultaneously resolve the same singleton provider for the first time
- **THEN** the provider SHALL be constructed exactly once

### Requirement: Container SHALL support transient provider registration and resolution
The DI container SHALL support registering a provider type as transient, where a new instance is constructed for each resolution.

#### Scenario: Transient returns new instance each time
- **WHEN** a provider is registered as transient and resolved multiple times
- **THEN** a NEW instance SHALL be returned for each resolution

### Requirement: Container SHALL support factory providers
The DI container SHALL support factory providers that receive the container and produce a value.

#### Scenario: Factory provider resolves correctly
- **WHEN** a factory provider is registered for a type
- **THEN** the factory SHALL be invoked during resolution and its result returned

### Requirement: Container SHALL detect circular dependencies
The DI container SHALL detect circular dependency chains during resolution and report them as errors.

#### Scenario: Direct circular dependency detected
- **WHEN** provider A depends on provider B and provider B depends on provider A
- **THEN** resolution SHALL fail with a circular dependency error

### Requirement: Container SHALL support provider overrides for testing
The DI container SHALL allow replacing a registered provider with an alternative implementation, scoped to the override context.

#### Scenario: Override replaces provider
- **WHEN** a provider is overridden in a test container
- **THEN** the override implementation SHALL be resolved instead of the original
