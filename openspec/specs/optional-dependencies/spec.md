## ADDED Requirements

### Requirement: `#[derive(Injectable)]` SHALL support optional fields
The `#[derive(Injectable)]` macro SHALL accept an `optional` attribute listing types that should be resolved optionally.

#### Scenario: Optional dependency resolves to Some
- **WHEN** a service struct has `#[injectable(optional = [Logger])]`
- **AND** a `Logger` provider is registered in the container
- **THEN** the field type SHALL be `Option<Logger>` and SHALL resolve to `Some(logger)`

#### Scenario: Optional dependency resolves to None
- **WHEN** a service struct has `#[injectable(optional = [Logger])]`
- **AND** no `Logger` provider is registered in the container
- **THEN** the field type SHALL be `Option<Logger>` and SHALL resolve to `None`

#### Scenario: Required dependency missing is an error
- **WHEN** a service struct has a required field (not in `optional` list)
- **AND** no corresponding provider is registered
- **THEN** the application SHALL fail to build with a missing provider error
