## ADDED Requirements

### Requirement: CLI library generator
The framework SHALL provide a CLI sub-generator for creating reusable library crates.

#### Scenario: Generate library
- **WHEN** `ironic generate library my-lib` is run
- **THEN** a new Cargo library crate is created with Ironic project structure and a `#[Module]` scaffold

#### Scenario: Library with module scaffold
- **WHEN** the generated library includes a module definition
- **THEN** it can be imported by other Ironic applications as a dependency
