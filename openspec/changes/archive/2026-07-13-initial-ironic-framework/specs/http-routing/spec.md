## ADDED Requirements

### Requirement: Framework SHALL define transport-neutral HTTP types
The framework SHALL define transport-neutral types for HTTP methods, status codes, requests, responses, and headers that abstract over the underlying HTTP implementation.

#### Scenario: HTTP method enum covers standard methods
- **WHEN** defining a route
- **THEN** the framework SHALL support GET, POST, PUT, PATCH, DELETE, HEAD, and OPTIONS methods

### Requirement: Route definitions SHALL include metadata
Each route SHALL carry metadata including HTTP method, path pattern, handler reference, guards, interceptors, and custom metadata.

#### Scenario: Route metadata is accessible
- **WHEN** a route is defined
- **THEN** its method, path, guards, and interceptors SHALL be inspectable

### Requirement: Framework SHALL support parameter extraction sources
The framework SHALL support extracting handler parameters from path segments, query strings, JSON body, headers, and request extensions.

#### Scenario: Path parameter is extracted
- **WHEN** a route has a path parameter `:id` and the handler declares `#[param] id: Uuid`
- **THEN** the parameter SHALL be parsed from the URL path segment

#### Scenario: JSON body is deserialized
- **WHEN** a handler declares `#[body] dto: CreateUserDto`
- **THEN** the request body SHALL be deserialized as JSON into the target type
