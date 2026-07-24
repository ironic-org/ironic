## MODIFIED Requirements

### Requirement: Framework SHALL provide MicroserviceClient and MicroserviceServer traits (replacing Transport trait)
**FROM:**
```rust
pub trait Transport: Send + Sync + 'static {
    fn send(&self, envelope: Envelope) -> TransportFuture<'_, ()>;
    fn receive(&self) -> TransportFuture<'_, Option<Envelope>>;
}
```
**TO:**
The framework SHALL provide `MicroserviceClient` and `MicroserviceServer` traits replacing the symmetric `Transport` trait. The old `Transport` trait SHALL be deprecated.

#### Scenario: MicroserviceClient sends request-response
- **WHEN** `client.send("pattern", data)` is called
- **THEN** the client serializes the payload, sends it with a correlation ID, awaits the reply, and deserializes the response

#### Scenario: MicroserviceClient emits event
- **WHEN** `client.emit("pattern", data)` is called
- **THEN** the client publishes the event without waiting for a response

#### Scenario: MicroserviceServer registers and dispatches handlers
- **WHEN** a server registers handlers via `on_message()` / `on_event()`
- **AND** `listen()` is called
- **THEN** the server listens for incoming messages and dispatches to the correct handler by pattern

#### Scenario: ChannelTransport migrated to new traits
- **WHEN** ChannelTransport is updated
- **THEN** it implements both `MicroserviceClient` and `MicroserviceServer` with the same in-memory channel semantics
