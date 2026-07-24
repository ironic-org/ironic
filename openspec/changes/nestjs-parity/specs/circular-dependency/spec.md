## ADDED Requirements

### Requirement: ForwardRef<T> for circular dependencies
The framework SHALL provide a `ForwardRef<T>` type that allows lazy resolution of circular dependencies between services.

#### Scenario: Circular dependency between two services
- **WHEN** Service A depends on Service B and Service B depends on Service A
- **THEN** both services can declare `ForwardRef<TheOtherService>`, and the DI container resolves both after construction

#### Scenario: Safe access with error handling
- **WHEN** `resolve()` is called on a `ForwardRef` before the container is fully built
- **THEN** it returns `Err` with a clear error message

### Requirement: #[forward_ref] attribute macro
The framework SHALL provide a `#[forward_ref]` attribute to mark constructor parameters as forward references.

#### Scenario: Mark parameter as forward ref
- **WHEN** `#[forward_ref] b: ForwardRef<ServiceB>` is used in a constructor
- **THEN** the DI container recognizes it as a circular dependency and constructs with lazy resolution
