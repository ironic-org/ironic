## ADDED Requirements

### Requirement: CLI script runner
The framework SHALL provide a CLI command for running custom scripts defined in the project.

#### Scenario: Define and run script
- **WHEN** a project defines `[package.metadata.ironic.scripts]` in Cargo.toml and `ironic run seed-db` is called
- **THEN** the CLI executes the corresponding Rust binary or shell command
