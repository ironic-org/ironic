## ADDED Requirements

### Requirement: Outbox record is enqueued atomically with business data

The `TransactionalOutbox` SHALL append an outbox record inside the same transaction as the business mutation, via an `enqueue(transaction, event)` API where `transaction` is a store-specific transaction handle.

#### Scenario: Enqueue inside a transaction

- **WHEN** a caller opens a transaction, mutates business state, and calls `TransactionalOutbox::enqueue(tx, event)` before committing
- **THEN** the outbox record SHALL be persisted as part of that transaction, committing or rolling back together with the business change

#### Scenario: Rollback discards the outbox record

- **WHEN** the enclosing transaction is rolled back
- **THEN** the outbox record SHALL NOT be published, because it was never committed

### Requirement: Outbox records carry routing metadata

An outbox record SHALL carry a unique message id, an event type string, and the serialized payload, matching the shape of `QueueMessage` envelopes for downstream compatibility.

#### Scenario: Record round-trips through serialization

- **WHEN** an outbox record is serialized and deserialized
- **THEN** all fields (id, event type, payload, status, attempt count) SHALL be preserved exactly

### Requirement: Relay publishes pending records at-least-once

An `OutboxRelay` background task SHALL periodically claim pending outbox records, publish each through a `RelaySink`, and mark them published only after the sink accepts the message. Failed publishes SHALL be retried with backoff, and records exceeding a maximum attempt count SHALL be marked dead-lettered.

#### Scenario: Relay publishes a pending record

- **WHEN** the relay polls and finds a pending record whose sink publish succeeds
- **THEN** the record SHALL be marked published and not be selected again

#### Scenario: Relay retries a failed publish

- **WHEN** a sink publish fails for a record with remaining attempts
- **THEN** the relay SHALL increment the attempt count, leave the record pending, and retry it on a later poll after a backoff delay

#### Scenario: Relay dead-letters after max attempts

- **WHEN** a record's attempt count reaches the configured maximum
- **THEN** the relay SHALL mark the record dead-lettered and stop retrying it

#### Scenario: Relay respects batch size

- **WHEN** more pending records exist than the configured batch size
- **THEN** the relay SHALL process at most `batch_size` records per poll cycle

### Requirement: Claiming prevents concurrent double-send

The store SHALL support a compare-and-set claim operation that transitions records from pending to claimed with an owner token and lease, so a batch is owned by at most one relay at a time.

#### Scenario: Lease expiry releases the claim

- **WHEN** a claimed record's lease expires without being published
- **THEN** the record SHALL become claimable again by the relay

### Requirement: OutboxStore is backend-neutral

The outbox SHALL be defined against an `OutboxStore` trait so that the in-memory store and future sqlx/seaorm/redis implementations are interchangeable without changing the relay logic.

#### Scenario: In-memory store satisfies the trait

- **WHEN** the in-memory store is substituted for any store behind the `OutboxStore` trait
- **THEN** the relay SHALL behave identically

### Requirement: Outbox is feature-gated and exported

The outbox capability SHALL be enabled only when the `outbox` feature flag is on, and its public types SHALL be exported from the framework prelude when that feature is enabled. All existing features SHALL still compile independently and together.

#### Scenario: Feature disabled hides outbox types

- **WHEN** the `outbox` feature is disabled
- **THEN** outbox types SHALL NOT be present in the prelude or public API

#### Scenario: All features compile together

- **WHEN** every feature flag is enabled at once
- **THEN** the workspace SHALL compile with `clippy::all` and `missing_docs` denials passing
