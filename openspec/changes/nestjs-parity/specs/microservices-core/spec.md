## ADDED Requirements

### Requirement: MicroserviceClient trait
The framework SHALL provide a `MicroserviceClient` trait for sending request-response messages and fire-and-forget events to remote services.

#### Scenario: Send request and receive response
- **WHEN** a service calls `client.send("user.get", GetUser { id: 1 }).await`
- **THEN** the client serializes the payload, sends it via the transport, matches the correlation ID on reply, deserializes the response, and returns the result

#### Scenario: Emit event without response
- **WHEN** a service calls `client.emit("user.created", UserEvent { id: 1 }).await`
- **THEN** the client serializes and publishes the event without waiting for a response

#### Scenario: Transport backend implements MicroserviceClient
- **WHEN** a transport backend (Redis, RabbitMQ, Kafka, TCP) implements `MicroserviceClient`
- **THEN** `send` and `emit` work with that backend's protocol

### Requirement: MicroserviceServer trait
The framework SHALL provide a `MicroserviceServer` trait for listening to incoming messages and dispatching to registered handlers.

#### Scenario: Register and handle message pattern
- **WHEN** a server registers `server.on_message("user.get", handler)` and a matching message arrives
- **THEN** the server deserializes the payload, calls the handler, serializes the response, and publishes it on the reply channel

#### Scenario: Register and handle event pattern
- **WHEN** a server registers `server.on_event("user.created", handler)` and a matching event arrives
- **THEN** the server deserializes the payload and calls the handler (no reply sent)

#### Scenario: Pattern mismatch returns error
- **WHEN** a message arrives with no matching handler
- **THEN** the server returns an error response with status "error" and message "NO_MESSAGE_HANDLER"

### Requirement: Correlation ID matching
The transport layer SHALL use correlation IDs to match request-response pairs across client and server.

#### Scenario: Response matched by correlation ID
- **WHEN** a client sends a message with a unique correlation ID and a response arrives with the same ID
- **THEN** the client dispatches the response to the correct pending request's completion handler

#### Scenario: Timeout for unmatched responses
- **WHEN** no response arrives within a configurable timeout
- **THEN** the client returns a timeout error to the caller
