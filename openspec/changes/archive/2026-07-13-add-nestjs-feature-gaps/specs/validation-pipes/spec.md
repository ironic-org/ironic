## ADDED Requirements

### Requirement: Framework SHALL provide pre-built validation pipes
The framework SHALL provide built-in `ParseIntPipe`, `ParseUUIDPipe`, `ParseBoolPipe`, and `ParseFloatPipe` that transform and validate string parameters.

#### Scenario: ParseIntPipe converts valid integer string
- **WHEN** a route parameter has `#[param(ParseIntPipe)]` and the value is `"42"`
- **THEN** the parameter SHALL be resolved as `42i64`

#### Scenario: ParseIntPipe rejects non-integer string
- **WHEN** a route parameter has `#[param(ParseIntPipe)]` and the value is `"abc"`
- **THEN** the handler SHALL NOT be invoked and a 400 Bad Request SHALL be returned with a parse error

#### Scenario: ParseUUIDPipe accepts valid UUID
- **WHEN** a route parameter has `#[param(ParseUUIDPipe)]` and the value is `"550e8400-e29b-41d4-a716-446655440000"`
- **THEN** the parameter SHALL be resolved as a `uuid::Uuid`

### Requirement: Framework SHALL integrate with `garde` for declarative validation
The framework SHALL provide a `ValidationPipe` that uses `garde`'s `Validate` derive macro to validate request bodies and query parameters.

#### Scenario: Valid body passes validation pipe
- **WHEN** a handler parameter has `#[body(ValidationPipe)]` with a type that derives `garde::Validate`
- **AND** the request body satisfies all validation rules
- **THEN** the parameter SHALL be resolved and the handler SHALL be invoked

#### Scenario: Invalid body fails validation pipe
- **WHEN** a handler parameter has `#[body(ValidationPipe)]` with a type that derives `garde::Validate`
- **AND** the request body violates a validation rule
- **THEN** the handler SHALL NOT be invoked and a 422 Unprocessable Entity SHALL be returned with validation error details

### Requirement: Pipes SHALL support global, controller, and parameter scope
Pipes SHALL be registrable at the application level (applied to all routes), controller level, and individual parameter level, with parameter-level pipes overriding controller-level and global pipes.

#### Scenario: Global pipe applies to all routes
- **WHEN** a global validation pipe is registered on the application
- **THEN** all route parameters SHALL be validated through that pipe by default
