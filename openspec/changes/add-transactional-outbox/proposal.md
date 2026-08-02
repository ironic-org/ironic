## Why

Apps built on ironic write business state to a DB (sqlx/seaorm/diesel/mongodb) and then publish events to a remote broker (Kafka/Redis/NATS via `transport_provider`, or `RedisQueue`) in separate, non-atomic operations. If the publish fails or the process crashes between the two writes, events are silently lost, leaving caches, search indexes, sagas, and downstream services stale. The transactional outbox pattern closes this dual-write gap by persisting outbound events in the same DB transaction as the business change and relaying them to the broker with at-least-once delivery.

## What Changes

- **`TransactionalOutbox` provider** in `ironic-distributed`: an injectable, lifecycle-managed service that appends outbox records in the same DB transaction as business data, then relays them to a broker asynchronously.
- **Relay poller** (`OutboxRelay`): a background task that polls unsent outbox records, publishes each to a `Queue`/transport destination, marks them sent only after successful publish, and retries on failure with a bounded batch size.
- **Inbox support** (`InboxConsumer`): idempotent-consumption helpers that deduplicate at-least-once deliveries via a processed-IDs table, so relayed events are handled exactly once.
- **In-memory backend** for local development/tests, plus a trait (`OutboxStore`) so sqlx/seaorm/redis implementations can be added without changing the relay logic.
- **Prelude exports** behind feature flags (`outbox`), gated so all features compile independently.
- **Docs + changelog**: new distributed/outbox.md page and changelog entries.

## Capabilities

### New Capabilities
- `outbox`: transactional outbox + relay — atomic enqueue of outbox records with business data, background relay with at-least-once publish, retry/backoff, dead-letter after max attempts
- `inbox`: idempotent consumer — dedupes at-least-once deliveries using a processed-message-id store

### Modified Capabilities
<!-- No existing specs in openspec/specs/ yet. -->

## Impact

- `crates/ironic-distributed/src/outbox.rs` (new) + `inbox.rs` (new) — core types, traits, in-memory store, relay task
- `crates/ironic-distributed/src/lib.rs` — module registration, feature gate `outbox`
- `src/lib.rs` — prelude exports when `outbox` feature enabled
- `Cargo.toml` — `outbox = []` feature flag
- `tests/extended_features.rs` — integration tests (relay publishes, retry on failure, dedupe)
- `CHANGELOG.md` — new entry
- Docs: `docs/content/docs/distributed/outbox.md`
- No breaking changes; all existing features compile independently and together
