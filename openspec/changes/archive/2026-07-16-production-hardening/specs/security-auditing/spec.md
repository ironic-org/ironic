# Security Auditing

## Purpose

Automated dependency vulnerability scanning and license compliance checking.

## Requirements

### Requirement: CI pipeline SHALL run cargo audit
The CI workflow SHALL run `cargo audit` to detect known vulnerabilities in dependencies.

#### Scenario: Vulnerability detected
- **WHEN** `cargo audit` finds a crate with a known advisory
- **THEN** the pipeline SHALL fail
- **AND** the advisory details SHALL be printed in the CI output

#### Scenario: No vulnerabilities found
- **WHEN** `cargo audit` completes with no advisories
- **THEN** the pipeline SHALL continue to the next step

### Requirement: CI pipeline SHALL run cargo deny
The CI workflow SHALL run `cargo deny check` to enforce license and duplicate-crate policies.

#### Scenario: License violation detected
- **WHEN** a dependency uses a license not in the allow list
- **THEN** the pipeline SHALL fail
- **AND** the violating crate and license SHALL be printed in the CI output

#### Scenario: Duplicate crate detected
- **WHEN** multiple versions of the same crate are in the dependency tree
- **THEN** the pipeline SHALL warn (not fail) and list the duplicates

### Requirement: deny.toml SHALL be checked into the repository
A `deny.toml` configuration file SHALL be committed to the repository root with license allow list, advisory severity thresholds, and duplicate detection settings.

#### Scenario: deny.toml exists
- **WHEN** `cargo deny check` is invoked
- **THEN** it SHALL read configuration from `deny.toml` at the repository root

### Requirement: audit script SHALL be available locally
A `scripts/audit.sh` script SHALL run `cargo audit` and `cargo deny` locally without CI.

#### Scenario: Local audit run
- **WHEN** a developer runs `scripts/audit.sh`
- **THEN** both `cargo audit` and `cargo deny check` SHALL execute
- **AND** results SHALL be printed to stdout
- **AND** the script SHALL exit non-zero if either check fails
