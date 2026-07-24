## ADDED Requirements

### Requirement: Cookie parsing
The framework SHALL support parsing cookies from incoming HTTP requests via a parameter decorator.

#### Scenario: Extract cookie value
- **WHEN** a handler parameter uses `#[cookie("session_id")] session_id: String`
- **THEN** the framework extracts the "session_id" cookie from the request headers

### Requirement: Cookie setting
The framework SHALL support setting cookies on outgoing HTTP responses.

#### Scenario: Set cookie in response
- **WHEN** a handler adds a cookie via the response builder
- **THEN** the `Set-Cookie` header is included in the response

### Requirement: Cookie decorator
The framework SHALL provide a `#[cookie]` parameter decorator for controller methods.

#### Scenario: Optional cookie
- **WHEN** a handler parameter uses `#[cookie("tracking")] tracking: Option<String>`
- **THEN** it extracts the cookie if present, returns `None` if absent
