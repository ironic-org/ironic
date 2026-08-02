## Context

ironic's distributed story (`crates/ironic-distributed`) provides transport-agnostic event clients/servers (`transport_provider.rs` → Kafka/Redis/NATS/MQTT/InMemory), a `Queue` trait with `RedisQueue`, and DB integrations (sqlx, seaorm, diesel, mongodb). The gap: publishing to a remote broker is a *separate* write from the DB transaction that owns the business change. A crash or transient broker outage between the two writes loses events permanently, or delivers them out of order with no retry story.

The transactional outbox pattern is the standard fix: the outbox row is written inside the *same DB transaction* as the business mutation, and a background relay guarantees at-least-once delivery to the broker. The inbox pattern complements it by making the consumer idempotent.

## Goals / Non-Goals

**Goals:**
- `TransactionalOutbox` injectable provider that appends outbox records atomically with business writes.
- `OutboxRelay` background task: poll → publish → mark-sent, with retry and backoff.
- `InboxConsumer` idempotency helper backed by a processed-IDs store.
- Storage abstraction (`OutboxStore`) with a working in-memory implementation; sqlx/seaorm adapters can be added later without touching the relay.
- Feature-gated (`outbox`), prelude exports, docs, changelog.

**Non-Goals:**
- No guarantee of exactly-once broker publish (impossible with at-least-once brokers) — idempotency is delegated to the inbox.
- No out-of-the-box sqlx/seaorm/redis store implementations in this change (trait + in-memory only).
- No ordering guarantees across partitions; ordering is per-store insertion order within a relay batch.
- No distributed locking/leader election for the relay (single-writer assumption, like the current `RedisQueue`).

## Decisions

### 1. Outbox rows are written in the user's own DB transaction
`TransactionalOutbox::enqueue(tx, event)` takes an already-open transaction-like handle. The framework cannot own the transaction because the business data and outbox row must commit atomically; the caller decides the transaction boundary.
- Alternative considered: framework-owned transaction scope (`outbox.in_tx(|| ...)`) — rejected: too invasive, couples the API to every DB driver's transaction type.
- Trade-off: each store adapter must define the "transaction handle" type; the in-memory backend treats a unit type `()` as its handle to keep the API usable for dev/tests.

### 2. Relay publishes through a `Queue`-style sink abstraction
The relay publishes each record to a `RelaySink` (a small `Send + Sync` trait wrapping `Queue::enqueue` and the transport `EventClient::emit`). This keeps the relay decoupled from Kafka vs Redis vs in-memory.
- Alternative: hard-code `EventClient` — rejected: `EventClient` is transport-specific and the `Queue` trait already exists and is backend-neutral.

### 3. Store marks records via compare-and-set on a status column
Records move `Pending → Published` (or `Dead` after max attempts). The relay claims a batch by marking `Pending → Claimed` with an owner token + lease, so a restarting relay doesn't double-send while another instance holds a lease.
- Alternative: delete-on-success (no status column) — rejected: loses the audit trail and makes retry/dead-letter bookkeeping impossible.
- Leases are advisory; with a single writer they never contend, but the schema supports it if a leader emerges later.

### 4. Inbox dedupes by a message-id primary key
`InboxConsumer::handle(message_id, f)` inserts the id into a `processed` store; if the insert conflicts, the handler is skipped. This gives at-most-once *handling* of at-least-once *delivery*.
- Alternative: content-hash dedup — rejected: message ids already exist in `QueueMessage.id` / event envelopes; a primary key insert is simpler and race-free.

### 5. Feature gating mirrors existing conventions
New `outbox = []` standalone feature in the root `Cargo.toml`, gate `#[cfg(feature = "outbox")]` in `ironic-distributed` (which is itself only reachable via the root feature), prelude exports only when enabled, `missing_docs`/`clippy::all` deny clean.

## Risks / Trade-offs

- [Relay marks Published only after sink accepts the message; if the sink succeeds but the DB update fails, the record is re-sent → duplicate delivery] → Mitigation: inbox dedup is the documented contract (at-least-once delivery, at-most-once handling).
- [In-memory store is not durable; a crash loses outbox rows] → Mitigation: in-memory is explicitly for tests/dev; production users must provide a durable `OutboxStore` (sqlx/seaorm adapter).
- [No leader election means two relays can claim the same batch] → Mitigation: lease token in the store's claim operation (CAS on status); documented single-writer assumption for v1.
- [Relay backoff tuning can starve low-throughput queues with fixed sleep] → Mitigation: `RelayConfig` exposes `poll_interval_ms` + `batch_size`; exponential backoff on repeated failures capped at a configurable max.

## Migration Plan

- Additive change: new module, trait, provider, feature flag. No existing API is modified or removed.
- Rollback: disable the `outbox` feature; existing distributed features compile independently.
- Docs page + changelog entry shipped in the same release.

## Open Questions

- Should the `OutboxRecord` envelope carry a typed `payload: Vec<u8>` only, or also `event_type: String` for routing on the consumer side? (Default: both, matching `QueueMessage` shape.)
- Should `TransactionalOutbox` expose a convenience `enqueue_many` for batch business writes? (Default: yes, one `enqueue` call per transaction, but the store trait supports batched insertion.)
