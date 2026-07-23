## ADDED Requirements

### Requirement: Framework SHALL provide `#[event_handler]` attribute macro
The framework SHALL provide an attribute macro that registers a method as an event listener on the `EventBus`. The event type SHALL be inferred from the method's single parameter type.

#### Scenario: EventHandler registers listener for inferred event type
- **WHEN** a method is annotated with `#[event_handler]`
- **AND** the method signature is `async fn handle_order_placed(event: Arc<OrderPlaced>)`
- **THEN** the method SHALL be registered as a listener for `OrderPlaced` events on the application's `EventBus`

#### Scenario: EventHandler method is invoked on matching event
- **WHEN** an `OrderPlaced` event is published to the `EventBus`
- **AND** a handler is registered for `OrderPlaced`
- **THEN** the handler method SHALL be invoked with the event

#### Scenario: Multiple event handlers for same event type
- **WHEN** two methods are annotated with `#[event_handler]` for the same event type
- **AND** an event of that type is published
- **THEN** both handlers SHALL be invoked

### Requirement: Framework SHALL support `#[event_handler(capacity = N)]` for backpressure
The `#[event_handler]` macro SHALL accept a `capacity` parameter that configures the bounded channel capacity for the subscriber.

#### Scenario: EventHandler capacity parameter configures backpressure
- **WHEN** a handler is annotated with `#[event_handler(capacity = 128)]`
- **THEN** the underlying `EventBus` subscription SHALL use a channel capacity of 128

### Requirement: Framework SHALL auto-register event handlers during module initialization
Event handlers defined in a module SHALL be automatically registered when the module is initialized by the DI container.

#### Scenario: EventHandler registered at module startup
- **WHEN** a module containing `#[event_handler]` methods is imported
- **AND** the application starts
- **THEN** all event handler methods in that module SHALL be subscribed to their respective event types

### Requirement: Framework SHALL gate `#[event_handler]` behind `events` feature
The `#[event_handler]` attribute macro SHALL only be available when `features = ["events"]` is enabled.

#### Scenario: event_handler not available without events feature
- **WHEN** `features = ["events"]` is not enabled
- **THEN** the `#[event_handler]` attribute SHALL produce a compile-time error
