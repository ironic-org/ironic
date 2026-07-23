# Developer Experience

## Purpose

Error backtraces in HttpError, CLI lint/debug commands, test fixture/ factory utilities.

## Requirements

### Requirement: Error backtraces in HttpError SHALL support optional backtrace capture
The framework SHALL capture a backtrace when `HttpError::internal()` is called and the `backtrace` feature is enabled. The `HttpError` type and all its public methods SHALL have full doc comments.

#### Scenario: Backtrace captured on internal error
- **WHEN** the `backtrace` feature is enabled
- **AND** `HttpError::internal("db connection failed")` is called
- **THEN** the error SHALL contain a backtrace pointing to the call site
- **AND** the backtrace SHALL be included in debug error responses

### Requirement: CLI SHALL provide lint command (P3)
The `ironic lint` command SHALL analyze source code for common Ironic-specific issues (unregistered providers, incorrect module wiring, deprecated API usage).

#### Scenario: Lint detects unregistered provider
- **WHEN** `ironic lint` is run on a project
- **AND** a provider is used but not registered in any module
- **THEN** the lint report SHALL include a warning with the file and line number

### Requirement: CLI SHALL provide debug command (P3)
The `ironic debug` command SHALL start an interactive REPL that allows inspecting the DI container, module graph, and loaded configuration at runtime. All public API items in the debug REPL SHALL have full doc comments.

#### Scenario: Debug inspects resolved providers
- **WHEN** `ironic debug` is running
- **AND** the user types `providers`
- **THEN** the REPL SHALL list all registered providers with their scopes and states

### Requirement: Testing SHALL support test fixture utilities (P2)
The framework SHALL provide helper utilities for creating test data, including a `FixtureBuilder` for constructing entities with sensible defaults. All public API items in the testing utilities SHALL have full doc comments.

#### Scenario: Fixture with overrides
- **WHEN** `FixtureBuilder::<User>::default().with(|u| u.name("test"))` is used
- **THEN** a `User` SHALL be created with default values except `name` which SHALL be `"test"`

### Requirement: All public APIs SHALL have doc comments
Every public function, struct, enum, trait, type alias, const, and macro in every crate SHALL have a `///` doc comment describing its purpose, arguments, return values, and (where applicable) panics and errors.

#### Scenario: Doc comment present on public function
- **WHEN** a user hovers over a public function name in their IDE
- **THEN** the doc comment SHALL be displayed

#### Scenario: Doc comment covers errors
- **WHEN** a public function returns a `Result` type
- **THEN** its doc comment SHALL include an `# Errors` section describing error conditions

#### Scenario: Doc comment covers panics
- **WHEN** a public function can panic
- **THEN** its doc comment SHALL include a `# Panics` section describing panic conditions

### Requirement: Crates SHALL deny missing docs via lint
Each crate that is not a proc-macro crate SHALL set `#![deny(missing_docs)]` in `lib.rs`.

#### Scenario: Undocumented public item fails build
- **WHEN** a public item lacks a doc comment
- **THEN** the build SHALL fail with a `missing_docs` lint error
