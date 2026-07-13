## ADDED Requirements

### Requirement: Framework SHALL support decorator-based WebSocket gateways
The framework SHALL provide a `#[WebSocketGateway]` attribute macro that registers a struct as a WebSocket endpoint.

#### Scenario: WebSocket gateway accepts connections
- **WHEN** a struct is annotated with `#[WebSocketGateway("/ws")]`
- **AND** a client connects to `/ws`
- **THEN** the gateway's connection handler SHALL be invoked

### Requirement: Framework SHALL support message routing via `#[SubscribeMessage]`
Gateway methods SHALL be annotatable with `#[SubscribeMessage("event_name")]` to handle specific message types.

#### Scenario: SubscribeMessage routes typed messages
- **WHEN** a gateway has a method annotated with `#[SubscribeMessage("chat.message")]`
- **AND** a client sends `{"event": "chat.message", "data": {...}}`
- **THEN** the annotated method SHALL be invoked with the deserialized data

#### Scenario: Unhandled message type is ignored
- **WHEN** a client sends a message with no matching `#[SubscribeMessage]` handler
- **THEN** the message SHALL be silently ignored

### Requirement: Framework SHALL support WebSocket rooms
Gateways SHALL support joining and leaving named rooms, and broadcasting messages to all clients in a room.

#### Scenario: Client joins room and receives broadcasts
- **WHEN** a client sends a `{"event": "room.join", "data": {"room": "general"}}`
- **AND** another client broadcasts to "general"
- **THEN** the first client SHALL receive the broadcast

#### Scenario: Client leaves room and stops receiving broadcasts
- **WHEN** a client sends a `{"event": "room.leave", "data": {"room": "general"}}`
- **AND** another client broadcasts to "general"
- **THEN** the first client SHALL NOT receive the broadcast

### Requirement: Framework SHALL support broadcasting to all connected clients
Gateways SHALL support server-side broadcasting to all connected clients, all clients in a room, or a specific client.

#### Scenario: Server broadcasts to all clients
- **WHEN** the server calls `broadcast_all(event, data)`
- **THEN** all connected WebSocket clients SHALL receive the message
