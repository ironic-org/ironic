## ADDED Requirements

### Requirement: Inbox consumer deduplicates at-least-once deliveries

The `InboxConsumer` SHALL handle each message at most once by recording processed message ids in a durable store before or atomically with running the handler. A duplicate delivery of an already-processed id SHALL be skipped.

#### Scenario: First delivery runs the handler

- **WHEN** a message with a new message id is delivered to the inbox consumer
- **THEN** the handler SHALL run and the message id SHALL be recorded as processed

#### Scenario: Duplicate delivery is skipped

- **WHEN** a message with an already-processed message id is delivered again
- **THEN** the handler SHALL NOT run again for that message

### Requirement: Inbox dedup store is backend-neutral

The inbox SHALL be defined against a `ProcessedStore` trait so the in-memory store and future durable implementations are interchangeable without changing consumer logic.

#### Scenario: In-memory store satisfies the trait

- **WHEN** the in-memory store is substituted for any store behind the `ProcessedStore` trait
- **THEN** deduplication SHALL behave identically

### Requirement: Inbox is feature-gated and exported

The inbox capability SHALL be enabled only when the `outbox` feature flag is on, and its public types SHALL be exported from the framework prelude when that feature is enabled.

#### Scenario: Feature disabled hides inbox types

- **WHEN** the `outbox` feature is disabled
- **THEN** inbox types SHALL NOT be present in the prelude or public API
