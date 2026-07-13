## ADDED Requirements

### Requirement: Framework SHALL support exception filters
The framework SHALL provide an `ExceptionFilter<E>` trait that allows users to catch specific exception types and return custom responses.

#### Scenario: Custom exception filter catches typed error
- **WHEN** a `UserNotFoundError` is thrown from a handler
- **AND** an `ExceptionFilter<UserNotFoundError>` is registered on the route or globally
- **THEN** the filter's `catch` method SHALL be invoked and its response SHALL be returned

#### Scenario: Unhandled exception falls through to default handler
- **WHEN** an exception type has no registered filter
- **THEN** the default error response SHALL be returned

### Requirement: Exception filters SHALL support global, controller, and route scope
Exception filters SHALL be registrable at the application level, controller level, and route level, with the most specific scope winning.

#### Scenario: Route-level filter overrides controller-level filter
- **WHEN** a controller registers a general `ExceptionFilter<AppError>`
- **AND** a specific route registers an `ExceptionFilter<AppError::NotFound>`
- **THEN** the route-level filter SHALL handle `NotFound` errors

### Requirement: Exception filters SHALL have access to request context
The filter's `catch` method SHALL receive a `FilterContext` containing the request, route metadata, and a reference to the DI container.

#### Scenario: Filter uses request context for logging
- **WHEN** an exception filter catches an error
- **THEN** it SHALL have access to the request ID, path, and method from `FilterContext`
