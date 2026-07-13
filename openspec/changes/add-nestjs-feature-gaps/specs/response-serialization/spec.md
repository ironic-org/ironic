## ADDED Requirements

### Requirement: Framework SHALL support field-level response exclusion
A `#[serde(skip_serializing_if)]`-like attribute SHALL allow fields to be conditionally excluded from JSON responses.

#### Scenario: Excluded field is omitted from response
- **WHEN** a response DTO has a field annotated with `#[exclude]`
- **THEN** that field SHALL NOT appear in the serialized JSON response

### Requirement: Framework SHALL support conditional field exposure
A `#[expose]` attribute SHALL allow fields to be included only when a specific role or condition is met.

#### Scenario: Exposed field included for authorized users
- **WHEN** a response DTO has a field annotated with `#[expose(role = "admin")]`
- **AND** the current user has the "admin" role
- **THEN** the field SHALL appear in the serialized JSON response

#### Scenario: Exposed field excluded for unauthorized users
- **WHEN** a response DTO has a field annotated with `#[expose(role = "admin")]`
- **AND** the current user does NOT have the "admin" role
- **THEN** the field SHALL NOT appear in the serialized JSON response

### Requirement: Framework SHALL provide a response serialization interceptor
A `SerializeInterceptor` SHALL apply field exclusion/exposure rules automatically to all handler responses.

#### Scenario: SerializeInterceptor transforms response
- **WHEN** a handler returns a response DTO with `#[exclude]` or `#[expose]` annotations
- **AND** the `SerializeInterceptor` is registered globally or on the route
- **THEN** the interceptor SHALL transform the response before serialization
