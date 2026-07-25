## ADDED Requirements

### Requirement: #[event_handler] SHALL support cross-process transport routing
The `#[event_handler]` macro SHALL optionally register handlers on a `MicroserviceServer` transport, not just the in-process `EventBus`.

#### Scenario: EventHandler with transport attribute
- **WHEN** `#[event_handler(transport = "redis")]` is used
- **THEN** the handler is registered on the Redis microservice server for the event type pattern, receiving events published by remote services

#### Scenario: EventHandler defaults to in-process
- **WHEN** `#[event_handler]` is used without `transport`
- **THEN** the handler is registered on the in-process `EventBus` (existing behavior preserved)
