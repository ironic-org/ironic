//! Transactional outbox with a background relay for at-least-once event delivery.
//!
//! The outbox closes the dual-write gap between a business DB transaction and a
//! remote broker publish: [`TransactionalOutbox::enqueue`] appends an outbox
//! record inside the caller's transaction, and [`OutboxRelay`] polls pending
//! records, publishes them through a [`RelaySink`], and marks them sent only
//! after the sink accepts the message. Failed publishes are retried with
//! exponential backoff and dead-lettered after a configurable attempt cap.
//!
//! Storage is backend-neutral through the [`OutboxStore`] trait; an
//! [`InMemoryOutboxStore`] is provided for tests and development, and durable
//! sqlx/seaorm/redis stores can be added without changing the relay logic.

use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::sync::watch;

#[cfg(feature = "queues")]
use super::queues::{Queue, QueueMessage};
use crate::{
    Dependency, LifecycleFuture, OnApplicationBootstrap, OnApplicationShutdown, ProviderDefinition,
    Scope, ShutdownSignal,
};

/// A failure in outbox storage or relay publishing.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("IRONIC_OUTBOX: {0}")]
pub struct OutboxError(pub String);

/// Boxed outbox operation.
pub type OutboxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, OutboxError>> + Send + 'a>>;

/// Delivery state of an outbox record.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OutboxStatus {
    /// Written inside a transaction, not yet claimed by the relay.
    Pending,
    /// Claimed by a relay with an active lease.
    Claimed,
    /// Successfully published and acknowledged by the sink.
    Published,
    /// Exhausted all delivery attempts.
    Dead,
}

/// A single outbound event persisted alongside business data.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutboxRecord {
    /// Unique message identifier used for downstream deduplication.
    pub id: String,
    /// Routing type for the consumer (e.g. `"order.created"`).
    pub event_type: String,
    /// Serialized event payload.
    pub payload: Vec<u8>,
    /// Current delivery state.
    pub status: OutboxStatus,
    /// Number of delivery attempts that have failed.
    pub attempt_count: u32,
    /// Unix epoch milliseconds when the record was created.
    pub created_at_ms: u64,
}

static RECORD_SEQ: AtomicU64 = AtomicU64::new(0);

fn next_record_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{millis}-{}", RECORD_SEQ.fetch_add(1, Ordering::Relaxed))
}

/// A destination that accepts published outbox records.
pub trait RelaySink: Send + Sync + 'static {
    /// Publishes a record, resolving `Ok` only when the destination accepted it.
    fn publish(&self, record: &OutboxRecord) -> OutboxFuture<'_, ()>;
}

/// An in-memory [`RelaySink`] that records published message ids.
///
/// Useful for tests, development, and as the default sink when no transport is
/// configured.
#[derive(Clone, Default)]
pub struct InMemorySink {
    published: Arc<StdMutex<Vec<String>>>,
}

impl InMemorySink {
    /// Creates an empty in-memory sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the ids of all records published so far.
    #[must_use]
    pub fn published(&self) -> Vec<String> {
        self.published
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl RelaySink for InMemorySink {
    fn publish(&self, record: &OutboxRecord) -> OutboxFuture<'_, ()> {
        let published = self.published.clone();
        let id = record.id.clone();
        Box::pin(async move {
            published
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(id);
            Ok(())
        })
    }
}

/// A [`RelaySink`] that enqueues records onto an at-least-once [`Queue`].
#[cfg(feature = "queues")]
#[derive(Clone)]
pub struct QueueSink {
    queue: Arc<dyn Queue>,
}

#[cfg(feature = "queues")]
impl QueueSink {
    /// Creates a sink backed by `queue`.
    #[must_use]
    pub fn new(queue: Arc<dyn Queue>) -> Self {
        Self { queue }
    }
}

#[cfg(feature = "queues")]
impl RelaySink for QueueSink {
    fn publish(&self, record: &OutboxRecord) -> OutboxFuture<'_, ()> {
        let queue = self.queue.clone();
        let record = record.clone();
        Box::pin(async move {
            let message = QueueMessage {
                id: record.id,
                headers: BTreeMap::from([("event_type".into(), record.event_type)]),
                payload: record.payload,
                retry_count: 0,
                max_retries: 0,
                ttl_secs: None,
            };
            queue.enqueue(message).await.map_err(|e| OutboxError(e.0))
        })
    }
}

/// Backend-neutral storage for outbox records.
///
/// Implementations define their own [`Transaction`](Self::Transaction) handle so
/// that [`TransactionalOutbox::enqueue`] can append a record inside the same
/// transaction as the business change. The in-memory store uses `()` as its
/// handle for development and tests.
pub trait OutboxStore: Send + Sync + 'static {
    /// The transaction handle type for this store.
    type Transaction: Send + Sync + 'static;

    /// Appends `record` as part of `transaction`.
    fn enqueue(
        &self,
        transaction: &Self::Transaction,
        record: OutboxRecord,
    ) -> OutboxFuture<'_, ()>;

    /// Claims up to `batch_size` pending records with an exclusive `owner` lease.
    ///
    /// Records that are `Pending`, or `Claimed` with an expired lease, are
    /// transitioned to `Claimed` and returned.
    fn claim_batch(
        &self,
        batch_size: usize,
        lease_secs: u64,
        owner: &str,
    ) -> OutboxFuture<'_, Vec<OutboxRecord>>;

    /// Marks a record as successfully published.
    fn mark_published(&self, id: &str) -> OutboxFuture<'_, ()>;

    /// Marks a record as dead-lettered after exhausting delivery attempts.
    fn mark_dead(&self, id: &str) -> OutboxFuture<'_, ()>;

    /// Releases a claim, leaving the record pending with an updated attempt count.
    fn release_claim(&self, id: &str, attempt_count: u32) -> OutboxFuture<'_, ()>;
}

/// An [`OutboxStore`] kept entirely in memory.
///
/// **Not durable** — intended for tests and development only. Production
/// deployments must supply a durable [`OutboxStore`] implementation.
#[derive(Clone, Default)]
pub struct InMemoryOutboxStore {
    records: Arc<StdMutex<BTreeMap<String, StoredRecord>>>,
}

/// A record plus its claim bookkeeping.
struct StoredRecord {
    record: OutboxRecord,
    claimed_by: Option<String>,
    claimed_until_ms: u64,
}

impl InMemoryOutboxStore {
    /// Creates an empty in-memory outbox store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current status of a record, if present.
    #[must_use]
    pub fn status(&self, id: &str) -> Option<OutboxStatus> {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .map(|stored| stored.record.status.clone())
    }

    /// Returns the current delivery attempt count of a record, if present.
    #[must_use]
    pub fn attempts(&self, id: &str) -> Option<u32> {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .map(|stored| stored.record.attempt_count)
    }

    /// Returns the number of records currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Returns whether the store contains no records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn now_ms() -> u64 {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u128::from(u64::MAX)),
    )
    .unwrap_or(u64::MAX)
}

impl OutboxStore for InMemoryOutboxStore {
    type Transaction = ();

    fn enqueue(
        &self,
        _transaction: &Self::Transaction,
        record: OutboxRecord,
    ) -> OutboxFuture<'_, ()> {
        let records = self.records.clone();
        Box::pin(async move {
            records
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(
                    record.id.clone(),
                    StoredRecord {
                        record,
                        claimed_by: None,
                        claimed_until_ms: 0,
                    },
                );
            Ok(())
        })
    }

    fn claim_batch(
        &self,
        batch_size: usize,
        lease_secs: u64,
        owner: &str,
    ) -> OutboxFuture<'_, Vec<OutboxRecord>> {
        let records = self.records.clone();
        let owner = owner.to_owned();
        Box::pin(async move {
            let mut guard = records
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let now = now_ms();
            let lease_ms = lease_secs.saturating_mul(1000);
            let mut claimed = Vec::new();
            let ids: Vec<String> = guard.keys().cloned().collect();
            for id in ids {
                if claimed.len() >= batch_size {
                    break;
                }
                let claimable = match guard.get(&id) {
                    Some(stored) => {
                        matches!(stored.record.status, OutboxStatus::Pending)
                            || (matches!(stored.record.status, OutboxStatus::Claimed)
                                && stored.claimed_until_ms <= now)
                    }
                    None => false,
                };
                if claimable && let Some(stored) = guard.get_mut(&id) {
                    stored.record.status = OutboxStatus::Claimed;
                    stored.claimed_by = Some(owner.clone());
                    stored.claimed_until_ms = now.saturating_add(lease_ms);
                    claimed.push(stored.record.clone());
                }
            }
            Ok(claimed)
        })
    }

    fn mark_published(&self, id: &str) -> OutboxFuture<'_, ()> {
        let records = self.records.clone();
        let id = id.to_owned();
        Box::pin(async move {
            if let Some(stored) = records
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_mut(&id)
            {
                stored.record.status = OutboxStatus::Published;
                stored.claimed_by = None;
            }
            Ok(())
        })
    }

    fn mark_dead(&self, id: &str) -> OutboxFuture<'_, ()> {
        let records = self.records.clone();
        let id = id.to_owned();
        Box::pin(async move {
            if let Some(stored) = records
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_mut(&id)
            {
                stored.record.status = OutboxStatus::Dead;
                stored.claimed_by = None;
            }
            Ok(())
        })
    }

    fn release_claim(&self, id: &str, attempt_count: u32) -> OutboxFuture<'_, ()> {
        let records = self.records.clone();
        let id = id.to_owned();
        Box::pin(async move {
            if let Some(stored) = records
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_mut(&id)
            {
                stored.record.status = OutboxStatus::Pending;
                stored.record.attempt_count = attempt_count;
                stored.claimed_by = None;
                stored.claimed_until_ms = 0;
            }
            Ok(())
        })
    }
}

/// An injectable producer that appends outbox records inside a transaction.
///
/// Register `TransactionalOutbox::<InMemoryOutboxStore>::provider_definition()`
/// as a provider to obtain a singleton backed by the in-memory store. Call
/// [`enqueue`](Self::enqueue) with the caller's transaction handle and event
/// before committing; the record commits or rolls back atomically with the
/// business change.
#[derive(Clone)]
pub struct TransactionalOutbox<S> {
    store: Arc<S>,
}

impl<S: OutboxStore> TransactionalOutbox<S> {
    /// Creates an outbox backed by `store`.
    #[must_use]
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    /// Returns a reference to the underlying store.
    #[must_use]
    pub fn store(&self) -> Arc<S> {
        self.store.clone()
    }

    /// Appends an outbox record as part of `transaction`.
    ///
    /// A unique message id is generated and returned. The record is not
    /// published until the relay picks it up after the transaction commits.
    ///
    /// # Errors
    ///
    /// Returns [`OutboxError`] when the store rejects the enqueue.
    pub async fn enqueue(
        &self,
        transaction: &S::Transaction,
        event_type: impl Into<String>,
        payload: impl Into<Vec<u8>>,
    ) -> Result<String, OutboxError> {
        let record = OutboxRecord {
            id: next_record_id(),
            event_type: event_type.into(),
            payload: payload.into(),
            status: OutboxStatus::Pending,
            attempt_count: 0,
            created_at_ms: now_ms(),
        };
        let id = record.id.clone();
        self.store.enqueue(transaction, record).await?;
        Ok(id)
    }
}

impl TransactionalOutbox<InMemoryOutboxStore> {
    /// Creates a [`ProviderDefinition`] backed by an in-memory store.
    ///
    /// Register with `#[module(providers = [TransactionalOutbox, ...])]` or the
    /// application builder.
    pub fn provider_definition() -> ProviderDefinition {
        ProviderDefinition::constructor::<Self, _>(Scope::Singleton, vec![], |_| {
            Ok(Self::new(Arc::new(InMemoryOutboxStore::new())))
        })
        .eager()
    }
}

/// Relay polling configuration.
#[derive(Clone, Debug)]
pub struct RelayConfig {
    /// Idle poll interval in milliseconds. Default: `1000`.
    pub poll_interval_ms: u64,
    /// Maximum records claimed per poll cycle. Default: `32`.
    pub batch_size: usize,
    /// Delivery attempts before a record is dead-lettered. Default: `3`.
    pub max_attempts: u32,
    /// Claim lease duration in seconds before a record is re-claimable. Default: `60`.
    pub lease_secs: u64,
    /// Base backoff in milliseconds for repeated poll failures. Default: `200`.
    pub backoff_base_ms: u64,
    /// Upper bound for exponential backoff in milliseconds. Default: `5000`.
    pub backoff_max_ms: u64,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            batch_size: 32,
            max_attempts: 3,
            lease_secs: 60,
            backoff_base_ms: 200,
            backoff_max_ms: 5000,
        }
    }
}

/// A background relay that publishes pending outbox records to a sink.
///
/// One relay should be constructed per outbox store. It claims batches, publishes
/// each record through the configured [`RelaySink`], marks records published only
/// after the sink succeeds, retries failures with exponential backoff, and
/// dead-letters records that exceed the configured attempt cap.
#[derive(Clone)]
pub struct OutboxRelay<S> {
    store: Arc<S>,
    sink: Arc<dyn RelaySink>,
    config: RelayConfig,
    owner: Arc<str>,
    stop_tx: Arc<StdMutex<Option<watch::Sender<bool>>>>,
}

static RELAY_SEQ: AtomicU64 = AtomicU64::new(0);

impl<S: OutboxStore> OutboxRelay<S> {
    /// Creates a relay that publishes records from `store` through `sink`.
    #[must_use]
    pub fn new(store: Arc<S>, sink: Arc<dyn RelaySink>, config: RelayConfig) -> Self {
        let owner: Arc<str> = format!(
            "relay-{}-{}",
            std::process::id(),
            RELAY_SEQ.fetch_add(1, Ordering::Relaxed)
        )
        .into();
        Self {
            store,
            sink,
            config,
            owner,
            stop_tx: Arc::new(StdMutex::new(None)),
        }
    }

    /// Processes a single batch and returns the number of records handled.
    ///
    /// A record is published, marked published, retried, or dead-lettered as
    /// appropriate. This is the body of [`run`](Self::run) and is exposed for
    /// manual or test-driven polling.
    ///
    /// # Errors
    ///
    /// Returns [`OutboxError`] when the store fails to claim or update records.
    pub async fn poll_once(&self) -> Result<usize, OutboxError> {
        let batch = self
            .store
            .claim_batch(self.config.batch_size, self.config.lease_secs, &self.owner)
            .await?;
        let mut handled = 0;
        for record in batch {
            match self.sink.publish(&record).await {
                Ok(()) => {
                    self.store.mark_published(&record.id).await?;
                }
                Err(_) if record.attempt_count + 1 >= self.config.max_attempts => {
                    self.store.mark_dead(&record.id).await?;
                }
                Err(_) => {
                    self.store
                        .release_claim(&record.id, record.attempt_count + 1)
                        .await?;
                }
            }
            handled += 1;
        }
        Ok(handled)
    }

    /// Runs the relay loop until `shutdown` resolves to `true` or is dropped.
    ///
    /// Polls [`poll_once`](Self::poll_once), sleeping the configured poll
    /// interval between idle cycles and applying exponential backoff on repeated
    /// poll failures.
    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) {
        let mut consecutive_failures: u32 = 0;
        loop {
            let poll = self.poll_once().await;
            match poll {
                Ok(_) => consecutive_failures = 0,
                Err(e) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    tracing::warn!(error = %e, "outbox relay poll failed");
                }
            }
            let delay_ms = if consecutive_failures == 0 {
                self.config.poll_interval_ms
            } else {
                let exponent = consecutive_failures.saturating_sub(1).min(16);
                self.config
                    .backoff_base_ms
                    .saturating_mul(1_u64 << exponent)
                    .min(self.config.backoff_max_ms)
            };
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        break;
                    }
                }
                () = tokio::time::sleep(Duration::from_millis(delay_ms)) => {}
            }
        }
    }

    /// Returns a reference to the underlying store.
    #[must_use]
    pub fn store(&self) -> Arc<S> {
        self.store.clone()
    }
}

impl OutboxRelay<InMemoryOutboxStore> {
    /// Creates a [`ProviderDefinition`] for a relay over the default in-memory store.
    ///
    /// Injects [`TransactionalOutbox`](TransactionalOutbox::<InMemoryOutboxStore>)
    /// for its store, an optional [`RelaySink`], and an optional [`RelayConfig`].
    /// Registering your own `Arc<dyn RelaySink>` and `RelayConfig` providers
    /// overrides the defaults.
    pub fn provider_definition() -> ProviderDefinition {
        ProviderDefinition::factory::<Self, _, _>(
            Scope::Singleton,
            vec![
                Dependency::required::<TransactionalOutbox<InMemoryOutboxStore>>(),
                Dependency::optional::<Arc<dyn RelaySink>>(),
                Dependency::optional::<RelayConfig>(),
            ],
            |resolver| async move {
                let outbox: Arc<TransactionalOutbox<InMemoryOutboxStore>> =
                    resolver.resolve().await?;
                let sink: Arc<dyn RelaySink> =
                    match resolver.resolve_optional::<Arc<dyn RelaySink>>().await? {
                        Some(arc_sink) => (*arc_sink).clone(),
                        None => Arc::new(InMemorySink::new()),
                    };
                let config: Option<Arc<RelayConfig>> = resolver.resolve_optional().await?;
                Ok(Self::new(
                    outbox.store(),
                    sink,
                    config.map_or_else(RelayConfig::default, |cfg| (*cfg).clone()),
                ))
            },
        )
        .eager()
    }
}

impl OnApplicationBootstrap for OutboxRelay<InMemoryOutboxStore> {
    fn on_application_bootstrap(&self) -> LifecycleFuture<'_> {
        let relay = self.clone();
        Box::pin(async move {
            let (tx, rx) = watch::channel(false);
            *relay
                .stop_tx
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(tx);
            tokio::task::spawn(async move { relay.run(rx).await });
            tracing::info!("outbox relay started");
            Ok(())
        })
    }
}

impl OnApplicationShutdown for OutboxRelay<InMemoryOutboxStore> {
    fn on_application_shutdown(&self, _signal: ShutdownSignal) -> LifecycleFuture<'_> {
        let stop_tx = self.stop_tx.clone();
        Box::pin(async move {
            if let Some(tx) = stop_tx
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            {
                let _ = tx.send(true);
            }
            tracing::info!("outbox relay stopped");
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record(id: &str) -> OutboxRecord {
        OutboxRecord {
            id: id.into(),
            event_type: "order.created".into(),
            payload: b"{\"id\":1}".to_vec(),
            status: OutboxStatus::Pending,
            attempt_count: 0,
            created_at_ms: 0,
        }
    }

    #[test]
    fn record_serialization_roundtrip_preserves_fields() {
        let record = sample_record("rec-1");
        let bytes = serde_json::to_vec(&record).unwrap();
        let decoded: OutboxRecord = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(record, decoded);
        assert_eq!(decoded.event_type, "order.created");
        assert_eq!(decoded.status, OutboxStatus::Pending);
    }

    #[test]
    fn status_serialization_roundtrip() {
        for status in [
            OutboxStatus::Pending,
            OutboxStatus::Claimed,
            OutboxStatus::Published,
            OutboxStatus::Dead,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let decoded: OutboxStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, decoded);
        }
    }

    #[tokio::test]
    async fn enqueue_and_claim_returns_record() {
        let store = Arc::new(InMemoryOutboxStore::new());
        let outbox = TransactionalOutbox::new(store.clone());
        let id = outbox
            .enqueue(&(), "order.created", b"payload")
            .await
            .unwrap();
        let batch = store.claim_batch(10, 60, "relay-a").await.unwrap();
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].id, id);
        assert_eq!(store.status(&id), Some(OutboxStatus::Claimed));
    }

    #[tokio::test]
    async fn claim_respects_batch_size() {
        let store = Arc::new(InMemoryOutboxStore::new());
        let outbox = TransactionalOutbox::new(store.clone());
        for i in 0..5 {
            outbox
                .enqueue(&(), "event", format!("payload-{i}"))
                .await
                .unwrap();
        }
        let first = store.claim_batch(2, 60, "relay-a").await.unwrap();
        assert_eq!(first.len(), 2);
        let second = store.claim_batch(10, 60, "relay-b").await.unwrap();
        assert_eq!(second.len(), 3);
    }

    #[tokio::test]
    async fn expired_lease_is_reclaimable() {
        let store = Arc::new(InMemoryOutboxStore::new());
        let outbox = TransactionalOutbox::new(store.clone());
        let id = outbox.enqueue(&(), "event", "payload").await.unwrap();
        let first = store.claim_batch(10, 0, "relay-a").await.unwrap();
        assert_eq!(first.len(), 1);
        let second = store.claim_batch(10, 60, "relay-b").await.unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].id, id);
    }

    #[tokio::test]
    async fn published_records_are_not_reclaimed() {
        let store = Arc::new(InMemoryOutboxStore::new());
        let outbox = TransactionalOutbox::new(store.clone());
        let id = outbox.enqueue(&(), "event", "payload").await.unwrap();
        let batch = store.claim_batch(10, 60, "relay-a").await.unwrap();
        store.mark_published(&id).await.unwrap();
        drop(batch);
        let again = store.claim_batch(10, 60, "relay-a").await.unwrap();
        assert!(again.is_empty());
    }

    #[tokio::test]
    async fn relay_publishes_and_marks_published() {
        let store = Arc::new(InMemoryOutboxStore::new());
        let outbox = TransactionalOutbox::new(store.clone());
        let id = outbox.enqueue(&(), "event", "payload").await.unwrap();
        let sink = Arc::new(InMemorySink::new());
        let relay = OutboxRelay::new(
            store.clone(),
            sink.clone(),
            RelayConfig {
                max_attempts: 3,
                ..Default::default()
            },
        );
        let handled = relay.poll_once().await.unwrap();
        assert_eq!(handled, 1);
        assert_eq!(sink.published(), vec![id.clone()]);
        assert_eq!(store.status(&id), Some(OutboxStatus::Published));
    }

    #[tokio::test]
    async fn relay_retries_then_dead_letters_after_max_attempts() {
        struct FailingSink;
        impl RelaySink for FailingSink {
            fn publish(&self, _record: &OutboxRecord) -> OutboxFuture<'_, ()> {
                Box::pin(async { Err(OutboxError("sink down".into())) })
            }
        }
        let store = Arc::new(InMemoryOutboxStore::new());
        let outbox = TransactionalOutbox::new(store.clone());
        let id = outbox.enqueue(&(), "event", "payload").await.unwrap();
        let relay = OutboxRelay::new(
            store.clone(),
            Arc::new(FailingSink),
            RelayConfig {
                max_attempts: 3,
                ..Default::default()
            },
        );
        let _ = relay.poll_once().await.unwrap();
        assert_eq!(store.status(&id), Some(OutboxStatus::Pending));
        assert_eq!(store.attempts(&id), Some(1));
        let _ = relay.poll_once().await.unwrap();
        assert_eq!(store.attempts(&id), Some(2));
        let _ = relay.poll_once().await.unwrap();
        assert_eq!(store.status(&id), Some(OutboxStatus::Dead));
    }

    #[test]
    fn relay_config_defaults() {
        let config = RelayConfig::default();
        assert_eq!(config.poll_interval_ms, 1000);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.lease_secs, 60);
        assert_eq!(config.backoff_base_ms, 200);
        assert_eq!(config.backoff_max_ms, 5000);
    }

    #[tokio::test]
    async fn sink_records_published_ids() {
        let sink = InMemorySink::new();
        assert!(sink.published().is_empty());
        let record = sample_record("id-1");
        let sink_arc = Arc::new(sink);
        sink_arc.publish(&record).await.unwrap();
        assert_eq!(sink_arc.published(), vec!["id-1".to_owned()]);
    }
}
