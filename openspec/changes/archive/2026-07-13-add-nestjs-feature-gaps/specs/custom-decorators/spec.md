## ADDED Requirements

### Requirement: Framework SHALL provide a `create_param_decorator!` macro
The framework SHALL provide a declarative macro that allows users to define custom parameter decorators from an extraction function.

#### Scenario: Custom decorator extracts header value
- **WHEN** a user defines `create_param_decorator!(UserAgent, |req: &Request| { req.headers().get("user-agent").cloned() })`
- **AND** a handler includes `#[user_agent] agent: String`
- **THEN** the `User-Agent` header value SHALL be extracted into the `agent` parameter

#### Scenario: Custom decorator supports validation
- **WHEN** a user defines a custom decorator with a pipe
- **AND** the pipe validation fails
- **THEN** the handler SHALL NOT be invoked and a validation error SHALL be returned

### Requirement: Custom decorators SHALL integrate with the `#[routes]` macro
Custom decorators defined via `create_param_decorator!` SHALL be usable inside `#[routes]` annotated handler methods.

#### Scenario: Custom decorator in routes macro
- **WHEN** a handler method uses a custom decorator parameter
- **AND** the method is annotated with `#[routes]`
- **THEN** the custom decorator SHALL be processed and the extraction code SHALL be generated
