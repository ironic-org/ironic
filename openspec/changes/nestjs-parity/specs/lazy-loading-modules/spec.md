## ADDED Requirements

### Requirement: Lazy module loading
The framework SHALL support loading modules at runtime on demand, not just at application startup.

#### Scenario: Declare module as lazy
- **WHEN** a module is imported with `imports: [LazyModule<HeavyModule>]`
- **THEN** the `HeavyModule` is not loaded until first access

#### Scenario: Load lazy module on demand
- **WHEN** a service calls `module_ref.load::<HeavyModule>()`
- **THEN** the module's providers are resolved and added to the container at runtime

#### Scenario: Lazy module lifecycle
- **WHEN** a lazy module is loaded
- **THEN** its lifecycle hooks (OnModuleInit, etc.) are executed at that point
