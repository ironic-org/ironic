# CLI Tooling — Delta

## Modified Requirements

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
