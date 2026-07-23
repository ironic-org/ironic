## ADDED Requirements

### Requirement: Framework SHALL provide `#[sse]` route attribute macro
The framework SHALL provide an attribute macro that marks a route handler as an SSE endpoint.

#### Scenario: SSE route returns Server-Sent Events stream
- **WHEN** a handler is annotated with `#[sse]`
- **AND** a client connects to the route
- **THEN** the response SHALL have `Content-Type: text/event-stream`
- **AND** the handler SHALL be able to send events to the connected client

#### Scenario: SSE route supports path and method
- **WHEN** a handler is annotated with `#[sse("/events")]`
- **THEN** the route SHALL be registered at path `/events`
- **AND** it SHALL respond to GET requests

### Requirement: Framework SHALL provide `SseRoute` extractor for sending events
The framework SHALL provide an `SseRoute` type that can be injected into SSE handler functions, providing a `send()` method to push events to the connected client.

#### Scenario: SseRoute sends event to client
- **WHEN** an SSE handler has an `SseRoute` parameter
- **AND** `sse_route.send(Event::default().data("hello"))` is called
- **THEN** the event SHALL be delivered to the connected client

#### Scenario: SseRoute send is awaitable
- **WHEN** `sse_route.send(event).await` is called
- **THEN** it SHALL await until the event is buffered or the connection is closed

### Requirement: Framework SHALL support SSE reconnection via `Last-Event-ID`
The SSE integration SHALL respect the `Last-Event-ID` HTTP header for reconnection, replaying missed events from an in-memory event buffer.

#### Scenario: Client reconnects with Last-Event-ID
- **WHEN** a client reconnects with `Last-Event-ID: event-42`
- **AND** events after `event-42` are in the reconnection buffer
- **THEN** those events SHALL be replayed to the client

### Requirement: Framework SHALL provide `SseConfig` for endpoint configuration
The framework SHALL provide an `SseConfig` struct with configurable reconnection buffer size, default event ID prefix, and keep-alive interval.

#### Scenario: SseConfig sets reconnection buffer size
- **WHEN** `SseConfig` is created with `reconnect_buffer_size = 512`
- **THEN** the reconnection buffer SHALL store at most 512 events

### Requirement: Framework SHALL gate SSE behind `sse` feature flag
The `#[sse]` attribute and `SseRoute` type SHALL only be available when `features = ["sse"]` is enabled.

#### Scenario: SSE types not compiled without sse feature
- **WHEN** only `features = ["realtime"]` is enabled
- **THEN** `#[sse]` macro and `SseRoute` SHALL NOT be available

### Requirement: Framework SHALL integrate SSE routes into the platform adapter
The Axum platform adapter SHALL map `#[sse]` routes to Axum's SSE response type.

#### Scenario: SSE route registered via AxumAdapter
- **WHEN** an application uses `AxumAdapter` with SSE routes
- **THEN** the SSE routes SHALL be mounted and functional on the Axum router
