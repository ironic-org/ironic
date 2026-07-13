## ADDED Requirements

### Requirement: Framework SHALL support `forRoot()` and `forRootAsync()` module patterns
Modules SHALL support a `forRoot()` static method pattern for accepting configuration and returning a `ModuleDefinition`.

#### Scenario: forRoot configures module with static options
- **WHEN** a module implements `forRoot(config: Config) -> ModuleDefinition`
- **AND** it is imported via `Module::forRoot(DatabaseConfig::new())`
- **THEN** the module SHALL be configured with the provided options

#### Scenario: forRootAsync configures module with async options
- **WHEN** a module implements `forRootAsync(config_loader: impl Future<Output=Config>)`
- **AND** it is imported via `Module::forRootAsync(load_config())`
- **THEN** the module SHALL be configured after the async config is resolved

### Requirement: Framework SHALL support `@Global()` scope
The framework SHALL provide a `#[global]` attribute that marks a module's exported providers as globally visible without requiring explicit imports.

#### Scenario: Global module providers visible to all modules
- **WHEN** a module is annotated with `#[global]`
- **AND** it exports a `DatabaseConnection` provider
- **THEN** any module in the application SHALL be able to inject `DatabaseConnection` without importing the global module

### Requirement: Framework SHALL support `ModuleRef` for runtime provider access
The framework SHALL provide a `ModuleRef` service that allows runtime access to the DI container for lazy resolution and dynamic provider access.

#### Scenario: ModuleRef resolves a provider
- **WHEN** `ModuleRef` is injected into a service
- **AND** `module_ref.resolve::<T>()` is called
- **THEN** the provider for `T` SHALL be resolved from the application container
