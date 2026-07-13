## ADDED Requirements

### Requirement: Module SHALL define imports, providers, controllers, and exports
A module SHALL declare its imports (other modules), providers (services/repositories), controllers, and exports (visible providers).

#### Scenario: Module with imports compiles
- **WHEN** a module declares imports of other modules
- **THEN** the module compiler SHALL recursively traverse and include imported providers and controllers

#### Scenario: Module exports are accessible to importing modules
- **WHEN** module A exports a provider and module B imports module A
- **THEN** module B SHALL resolve providers exported by module A

### Requirement: Module compiler SHALL detect circular imports
The module compiler SHALL detect cycles in the module import graph and report them.

#### Scenario: Circular module import detected
- **WHEN** module A imports module B and module B imports module A
- **THEN** compilation SHALL fail with a circular import error

### Requirement: Module compiler SHALL compute deterministic initialization order
The module compiler SHALL produce a topologically sorted initialization order based on the import graph.

#### Scenario: Initialization order follows import dependencies
- **WHEN** module A imports module B
- **THEN** module B SHALL be initialized before module A

### Requirement: Module SHALL enforce provider visibility
Providers not listed in a module's exports SHALL NOT be resolvable from importing modules.

#### Scenario: Unexported provider is inaccessible
- **WHEN** a module does not export a provider and an importing module tries to resolve it
- **THEN** resolution SHALL fail with a visibility error
