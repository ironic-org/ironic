## 1. Feature gate and scaffolding

- [x] 1.1 Add `outbox = []` to root `Cargo.toml` feature flags
- [x] 1.2 Create `crates/ironic-distributed/src/outbox.rs` with `#[cfg(feature = "outbox")]` gating and module registration in `lib.rs`
- [x] 1.3 Add `outbox` module to `ironic-distributed` lib exports

## 2. Outbox core types

- [x] 2.1 Define `OutboxRecord` (id, event_type, payload, status, attempt_count, created_at) with `Serialize`/`Deserialize`
- [x] 2.2 Define `OutboxStatus` enum (`Pending`, `Claimed`, `Published`, `Dead`)
- [x] 2.3 Define `RelaySink` trait (async `publish(&OutboxRecord)`) with `Queue`-based impl and in-memory impl
- [x] 2.4 Define `OutboxStore` trait: `enqueue`, `claim_batch`, `mark_published`, `mark_dead`, `release_claim`
- [x] 2.5 Implement `InMemoryOutboxStore` (Mutex + Vec/BTreeMap, lease-expiry on claim)
- [x] 2.6 Define `TransactionalOutbox` with `enqueue(tx, event)` delegating to the store
- [x] 2.7 Define `RelayConfig` (poll_interval_ms, batch_size, max_attempts, backoff) with `Default`

## 3. Relay task

- [x] 3.1 Implement `OutboxRelay::run()` loop: claim batch → publish via sink → mark published/dead → sleep poll interval
- [x] 3.2 Implement exponential backoff on failed publishes capped by config
- [x] 3.3 Wire `TransactionalOutbox` + `OutboxRelay` as DI providers with lifecycle (start relay on bootstrap, stop on shutdown)

## 4. Inbox

- [x] 4.1 Define `ProcessedStore` trait (`is_processed`, `mark_processed`)
- [x] 4.2 Implement `InMemoryProcessedStore`
- [x] 4.3 Implement `InboxConsumer::handle(message_id, f)` with dedup-by-primary-key semantics

## 5. Exports and integration

- [x] 5.1 Export outbox/inbox types from `src/lib.rs` prelude behind `outbox` feature
- [x] 5.2 Add integration tests in `tests/extended_features.rs`: relay publishes, retry on failure, dead-letter after max attempts, dedup duplicate delivery
- [x] 5.3 Add unit tests in `outbox.rs`: serialization round-trip, lease expiry, batch-size cap, feature-disabled compilation
- [x] 5.4 Run `cargo fmt`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test` for all feature combinations

## 6. Docs and changelog

- [x] 6.1 Add `docs/content/docs/distributed/outbox.md` documenting outbox + inbox usage
- [x] 6.2 Add changelog entry via `./scripts/add-changelog-entry.sh Added "Transactional outbox and inbox..."`

## 7. Verification

- [x] 7.1 Verify all features compile independently and together with `cargo check --no-default-features` and `--all-features`
- [x] 7.2 Verify `missing_docs = "deny"` passes for all new public items