# CLI Tooling

## Purpose

Project scaffolding, code generators (module, controller, service, resource), build/start/test orchestration, doctor command.

## Requirements

### Requirement: CLI SHALL scaffold new projects
The `rustframe new <name>` command SHALL generate a complete Cargo project with the framework dependency, module structure, and a hello-world example.

#### Scenario: New project is generated
- **WHEN** `rustframe new my-api` is run
- **THEN** a Cargo project with `ironic` dependency SHALL be created at `./my-api`

### Requirement: CLI SHALL generate code components
The `generate` command SHALL support generating modules, controllers, services, and resources with the appropriate boilerplate.

#### Scenario: Module is generated
- **WHEN** `rustframe generate module users` is run
- **THEN** a module file with empty imports, providers, and exports SHALL be created

#### Scenario: Controller is generated
- **WHEN** `rustframe generate controller users` is run
- **THEN** a controller file with a placeholder route SHALL be created

### Requirement: CLI SHALL orchestrate build, start, and test
The CLI SHALL delegate build, run, and test operations to `cargo` while providing framework-aware output formatting.

#### Scenario: Start runs cargo
- **WHEN** `rustframe start` is run
- **THEN** the CLI SHALL execute `cargo run` in the project directory

#### Scenario: Test runs cargo test
- **WHEN** `rustframe test` is run
- **THEN** the CLI SHALL execute `cargo test` in the project directory

### Requirement: CLI SHALL provide a doctor command
The `doctor` command SHALL check the project environment including Rust version, framework version, project configuration, and platform adapter availability.

#### Scenario: Doctor checks environment
- **WHEN** `rustframe doctor` is run
- **THEN** the CLI SHALL report the status of Rust toolchain, project config, and framework compatibility
