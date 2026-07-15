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

### Requirement: CLI SHALL generate ready-resource components
The `generate` command SHALL support a `ready-resource` subcommand that generates production-ready modules with authentication, authorization, and full business logic.

#### Scenario: Ready resource auth is generated
- **WHEN** `ironic generate ready-resource auth` is run inside an Ironic project
- **THEN** a complete `src/modules/auth/` module SHALL be created with:
  - `AuthService` with register, login, refresh, me, logout methods
  - `PasswordService` with Argon2id hashing
  - `AuthController` at `/auth` with all routes
  - `User` entity with role enum
  - Role-based guards and custom decorators
  - Unit tests and integration tests
- **AND** `cargo test` SHALL pass immediately without additional setup

#### Scenario: Ready resource variants
- **WHEN** `ironic generate ready-resource auth-basic` is run
- **THEN** only password hashing and sessions SHALL be generated
- **WHEN** `ironic generate ready-resource auth-jwt` is run
- **THEN** only JWT token management SHALL be generated
- **WHEN** `ironic generate ready-resource auth-oauth` is run
- **THEN** only OAuth2 social login SHALL be generated

#### Scenario: Auto-registration
- **WHEN** any ready-resource variant is generated
- **THEN** the module SHALL be auto-registered in `src/modules/mod.rs`
- **AND** the module SHALL be auto-imported in `src/app.rs`
- **AND** the generated Cargo.toml SHALL include required dependencies
