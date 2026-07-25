## ADDED Requirements

### Requirement: GraphQL resolver decorator
The framework SHALL provide a `#[graphql_resolver]` proc-macro that marks a struct as a GraphQL resolver with DI injection.

#### Scenario: Define resolver with DI
- **WHEN** a struct is annotated with `#[graphql_resolver]` and has injectable fields
- **THEN** it is registered in the DI container and its fields are populated from the container

### Requirement: GraphQL query decorator
The framework SHALL provide a `#[graphql_query]` proc-macro that marks a method as a GraphQL query field.

#### Scenario: Simple query
- **WHEN** a method is annotated with `#[graphql_query]` on a resolver
- **THEN** it becomes a field in the GraphQL Query type, callable via GraphQL queries

### Requirement: GraphQL mutation decorator
The framework SHALL provide a `#[graphql_mutation]` proc-macro that marks a method as a GraphQL mutation.

#### Scenario: Mutation with input
- **WHEN** a method is annotated with `#[graphql_mutation]` accepting an input struct
- **THEN** it becomes a field in the GraphQL Mutation type, callable via GraphQL mutations

### Requirement: GraphQL subscription decorator
The framework SHALL provide a `#[graphql_subscription]` proc-macro that marks a method as a GraphQL subscription.

#### Scenario: Subscription with stream
- **WHEN** a method is annotated with `#[graphql_subscription]` returning a stream
- **THEN** it becomes a field in the GraphQL Subscription type

### Requirement: Schema merging
The framework SHALL automatically merge all resolvers from imported modules into a single GraphQL schema.

#### Scenario: Module-based resolver registration
- **WHEN** a module imports resolver A and resolver B
- **THEN** the resulting schema includes both resolvers' queries, mutations, and subscriptions

#### Scenario: Model sharing with HTTP
- **WHEN** a DTO is used in both a REST controller and a GraphQL resolver
- **THEN** the same Rust struct generates both JSON schema (OpenAPI) and GraphQL type definitions
