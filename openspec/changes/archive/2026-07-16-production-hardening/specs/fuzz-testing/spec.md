# Fuzz Testing

## Purpose

Continuous fuzz testing of HTTP request parsing paths to discover crash-inducing inputs, panics, and edge cases.

## Requirements

### Requirement: Fuzz target SHALL cover HTTP request parsing
A `cargo-fuzz` target SHALL be created that feeds raw bytes into the HTTP extraction pipeline (multipart, JSON body, URL query parameters, headers).

#### Scenario: Fuzz target runs
- **WHEN** `cargo fuzz run http_parse` is executed
- **THEN** the harness SHALL generate random byte sequences
- **AND** feed them through ironic-http's extraction logic
- **AND** report any panics or crashes

### Requirement: fuzz crate SHALL be in fuzz/ directory
The fuzz project SHALL live at `fuzz/Cargo.toml` with `cargo-fuzz` configuration and not interfere with the workspace build.

#### Scenario: Fuzz crate not built by default
- **WHEN** `cargo build` is run from the workspace root
- **THEN** the fuzz crate SHALL NOT be compiled
- **WHEN** `cargo fuzz build` is run from the repository root
- **THEN** the fuzz crate SHALL compile

### Requirement: Corpus SHALL include valid HTTP fixtures
The fuzz corpus SHALL contain a set of valid HTTP requests to guide the fuzzer toward meaningful inputs.

#### Scenario: Corpus seeded
- **WHEN** `cargo fuzz run http_parse` starts
- **THEN** it SHALL begin with the seeded corpus in `fuzz/corpus/http_parse/`
- **AND** any discovered crashing inputs SHALL be saved

### Requirement: CI SHALL run fuzz tests for a limited duration
The CI pipeline SHALL run the fuzz target for a fixed duration (60 seconds) to catch regressions without blocking the pipeline.

#### Scenario: Fuzz test in CI
- **WHEN** the CI pipeline reaches the fuzz step
- **THEN** `cargo fuzz run http_parse -- -max_total_time=60` SHALL execute
- **AND** the pipeline SHALL fail if a crash is found
