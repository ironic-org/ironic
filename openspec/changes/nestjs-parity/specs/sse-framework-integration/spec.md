## ADDED Requirements

### Requirement: Broadcast-based SSE via AxumAdapter::sse_route()
The framework SHALL provide `AxumAdapter::sse_route()` for broadcast-based SSE event distribution to multiple connected clients.

#### Scenario: Broadcast event to all SSE clients
- **WHEN** an event is sent via `broadcaster.send(event)`
- **THEN** all connected SSE clients receive the event

#### Scenario: New SSE client receives future events
- **WHEN** a client connects to an SSE endpoint after events have been sent
- **THEN** the client receives only events sent after its connection

### Requirement: EventBroadcaster type alias exported from prelude
The framework SHALL export `EventBroadcaster` as a type alias for `broadcast::Sender<Event>` from the prelude.

#### Scenario: EventBroadcaster injected into service
- **WHEN** a service injects `EventBroadcaster`
- **THEN** it can call `broadcaster.send(event)` to push events to all connected clients

### Requirement: SSE documentation
The SSE documentation page SHALL be added to `docs/content/docs/transport/meta.json` for sidebar visibility.

#### Scenario: SSE page appears in transport sidebar
- **WHEN** a user navigates to the Transport section in docs
- **THEN** "Server-Sent Events" appears in the sidebar navigation
