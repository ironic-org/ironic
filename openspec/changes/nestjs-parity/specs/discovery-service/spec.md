## ADDED Requirements

### Requirement: DiscoveryService
The framework SHALL provide a `DiscoveryService` that allows runtime inspection of registered providers, handlers, and modules.

#### Scenario: Discover all providers
- **WHEN** `discovery_service.get_providers()` is called
- **THEN** it returns a list of all registered providers with their types and scopes

#### Scenario: Find providers by metadata
- **WHEN** `discovery_service.find_providers_with_metadata::<CustomMeta>()` is called
- **THEN** it returns all providers that have the specified metadata attached

#### Scenario: Discover message handlers
- **WHEN** `discovery_service.get_message_handlers()` is called
- **THEN** it returns all registered `#[message_handler]` and `#[event_handler]` methods with their patterns
