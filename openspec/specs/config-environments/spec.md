# Configuration Environments

## Purpose

Environment-aware config profiles (dev/staging/prod), config hot reload, runtime feature toggles.

## Requirements

### Requirement: Configuration SHALL support environment profiles
The `ConfigurationLoader` SHALL auto-detect the active environment from `IRONIC_ENV` or `APP_ENV` environment variables and automatically load `config.{env}.toml` as an overlay on top of `config.toml`.

#### Scenario: Environment profile loaded
- **WHEN** `IRONIC_ENV=prod` is set
- **AND** `config.toml` has `host = "localhost"`
- **AND** `config.prod.toml` has `host = "0.0.0.0"`
- **THEN** the resolved `host` SHALL be `"0.0.0.0"`

#### Scenario: No environment file is optional
- **WHEN** `IRONIC_ENV` is not set
- **THEN** only `config.toml` SHALL be loaded
- **AND** no error SHALL be raised

### Requirement: Configuration SHALL support hot reload (P3)
The `ConfigurationLoader` SHALL support watching config files for changes and applying updates without restarting the process.

#### Scenario: Config file changed
- **WHEN** `config.toml` is modified on disk
- **THEN** a registered `on_reload` callback SHALL be invoked with the new configuration

### Requirement: System SHALL support runtime feature toggles (P3)
The framework SHALL provide a `FeatureToggle` provider that reads boolean flags from configuration and supports hot-reload.

#### Scenario: Feature toggle checked
- **WHEN** `config.toml` has `[features] new_checkout = true`
- **AND** application code calls `features.is_enabled("new_checkout")`
- **THEN** it SHALL return `true`
