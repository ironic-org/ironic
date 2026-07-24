## ADDED Requirements

### Requirement: Raw body accessor
The framework SHALL provide a way to access the raw request body before deserialization.

#### Scenario: Access raw body bytes
- **WHEN** a handler parameter uses `#[raw_body] body: Vec<u8>`
- **THEN** the raw request body bytes are injected into the parameter
