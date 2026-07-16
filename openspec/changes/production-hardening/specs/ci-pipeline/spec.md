# CI Pipeline

## Purpose

Automated GitHub Actions workflows for continuous integration and release publishing.

## Requirements

### Requirement: CI pipeline SHALL run on every PR and push to main
The CI workflow SHALL trigger on `pull_request` and `push` to `main` branch, running all verification steps.

#### Scenario: PR triggers CI
- **WHEN** a pull request is opened or updated
- **THEN** the CI workflow SHALL run all verification steps
- **AND** the commit status SHALL be reported back to the PR

### Requirement: CI pipeline SHALL verify code formatting
The CI workflow SHALL run `cargo fmt --all -- --check` and fail if formatting is incorrect.

#### Scenario: Format check fails
- **WHEN** `cargo fmt --check` detects unformatted code
- **THEN** the CI pipeline SHALL fail with a clear error message

### Requirement: CI pipeline SHALL run clippy with all features
The CI workflow SHALL run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

#### Scenario: Clippy lint violation
- **WHEN** clippy detects a lint violation
- **THEN** the CI pipeline SHALL fail with the lint details

### Requirement: CI pipeline SHALL run all tests with all features
The CI workflow SHALL run `cargo test --all-features`.

#### Scenario: Test suite passes
- **WHEN** `cargo test --all-features` completes
- **THEN** the CI output SHALL report pass/fail for each test
- **AND** a failure SHALL cause the pipeline to fail

### Requirement: CI pipeline SHALL build documentation
The CI workflow SHALL run `npm run build` in the `docs/` directory.

#### Scenario: Docs build succeeds
- **WHEN** `npm run build` completes in the docs directory
- **THEN** the built assets SHALL be available in `docs/dist/`
- **AND** a failure SHALL cause the pipeline to fail

### Requirement: Release workflow SHALL trigger on tag push
Pushing a tag matching `v*.*.*` SHALL trigger a release workflow.

#### Scenario: Tag triggers release
- **WHEN** a tag matching `/^v\d+\.\d+\.\d+$/` is pushed
- **THEN** the release workflow SHALL run verification steps
- **AND** the workflow SHALL create a GitHub Release with changelog

### Requirement: CI SHALL cache dependencies
The CI workflow SHALL cache `~/.cargo` and `target/` directories to speed up subsequent runs.

#### Scenario: Cache hit reduces build time
- **WHEN** the cache key for `Cargo.lock` matches a previous run
- **THEN** cached compiled artifacts SHALL be restored
- **AND** the build SHALL skip recompilation of unchanged crates
