## ADDED Requirements

### Requirement: Injectable derive macro SHALL generate Provider implementation
The `#[derive(Injectable)]` macro SHALL generate a `Provider` trait implementation that constructs the type by resolving its fields from the DI container.

#### Scenario: Injectable macro generates valid Provider
- **WHEN** a struct is annotated with `#[derive(Injectable)]`
- **THEN** a Provider impl SHALL be generated that constructs the struct via field-resolution

### Requirement: Module derive macro SHALL generate ModuleMetadata
The `#[derive(Module)]` macro SHALL generate a `module_definition()` function that returns a `ModuleMetadata` struct populated from attributes.

#### Scenario: Module macro generates metadata
- **WHEN** a struct is annotated with `#[derive(Module)]` and `#[module(...)]` attributes
- **THEN** a function returning `ModuleMetadata` with imports, controllers, providers, and exports SHALL be generated

### Requirement: Controller macro SHALL generate controller metadata
The `#[controller(path)]` derive macro SHALL generate metadata associating the controller with a URL path prefix.

#### Scenario: Controller macro registers path prefix
- **WHEN** a struct is annotated with `#[controller("/api/users")]`
- **THEN** route definitions within the controller SHALL be prefixed with `/api/users`

### Requirement: Route macros SHALL generate route registrations
`#[get]`, `#[post]`, `#[put]`, `#[patch]`, `#[delete]` macros SHALL generate route registration code within a `#[routes]` impl block.

#### Scenario: Route macro generates registration
- **WHEN** a method is annotated with `#[get("/:id")]` inside a `#[routes]` impl block
- **THEN** a `RouteDefinition` SHALL be generated matching GET method and the path pattern

### Requirement: Parameter attribute macros SHALL generate extraction code
`#[body]`, `#[param]`, `#[query]`, `#[header]` macros SHALL generate parameter extraction code for the annotated handler parameter.

#### Scenario: Body attribute generates deserialization
- **WHEN** a handler parameter is annotated with `#[body]`
- **THEN** extraction code SHALL deserialize the request body into the parameter type
