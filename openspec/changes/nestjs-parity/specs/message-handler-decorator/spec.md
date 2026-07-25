## ADDED Requirements

### Requirement: #[message_handler] proc-macro
The framework SHALL provide a `#[message_handler]` proc-macro that registers a method as a request-response message handler on the microservice server.

#### Scenario: Annotate method as message handler
- **WHEN** a method is annotated with `#[message_handler("user.get")]` on a controller
- **THEN** it is registered with the microservice server for the pattern "user.get", accepting deserialized payload and returning a response

#### Scenario: Method receives and returns typed data
- **WHEN** the handler method signature is `async fn get_user(&self, req: GetUserRequest) -> Result<GetUserResponse, Error>`
- **THEN** the framework deserializes incoming data to `GetUserRequest` and serializes the response from `GetUserResponse`

#### Scenario: Auto-registration with module
- **WHEN** a controller containing `#[message_handler]` methods is imported in a module
- **THEN** the message handlers are automatically registered when the microservice server starts

### Requirement: Request context injection
Message handlers SHALL receive execution context (pattern, transport type, correlation ID).

#### Scenario: Context parameter injection
- **WHEN** a handler method includes a `ctx: MessageContext` parameter
- **THEN** the framework injects the context containing the matched pattern, transport ID, and correlation ID
