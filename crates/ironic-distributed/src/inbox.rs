//! Idempotent consumption of at-least-once deliveries.
//!
//! Brokers deliver at-least-once, so consumers can see duplicate messages. The
//! inbox pattern records processed message ids in a [`ProcessedStore`] and
//! skips handlers for ids that have already been handled, giving at-most-once
//! *handling* of at-least-once *delivery*. The [`InMemoryProcessedStore`] is
//! provided for tests and development; durable implementations (sqlx, seaorm,
//! redis) can be added behind the same trait.

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex as StdMutex},
};

/// A failure in inbox deduplication or handler execution.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("IRONIC_INBOX: {0}")]
pub struct InboxError(pub String);

/// Boxed inbox operation.
pub type InboxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, InboxError>> + Send + 'a>>;

/// Backend-neutral storage of processed message ids.
pub trait ProcessedStore: Send + Sync + 'static {
    /// Returns whether `message_id` has already been handled.
    fn is_processed(&self, message_id: &str) -> InboxFuture<'_, bool>;
    /// Records `message_id` as handled.
    fn mark_processed(&self, message_id: &str) -> InboxFuture<'_, ()>;
}

/// An in-memory [`ProcessedStore`] backed by a hash set of message ids.
///
/// **Not durable** — intended for tests and development only.
#[derive(Clone, Default)]
pub struct InMemoryProcessedStore {
    processed: Arc<StdMutex<std::collections::HashSet<String>>>,
}

impl InMemoryProcessedStore {
    /// Creates an empty in-memory processed-id store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of recorded message ids.
    #[must_use]
    pub fn len(&self) -> usize {
        self.processed
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Returns whether no message ids have been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl ProcessedStore for InMemoryProcessedStore {
    fn is_processed(&self, message_id: &str) -> InboxFuture<'_, bool> {
        let processed = self.processed.clone();
        let message_id = message_id.to_owned();
        Box::pin(async move {
            Ok(processed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains(&message_id))
        })
    }

    fn mark_processed(&self, message_id: &str) -> InboxFuture<'_, ()> {
        let processed = self.processed.clone();
        let message_id = message_id.to_owned();
        Box::pin(async move {
            processed
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(message_id);
            Ok(())
        })
    }
}

/// An idempotent consumer that deduplicates at-least-once deliveries.
#[derive(Clone)]
pub struct InboxConsumer<S> {
    store: Arc<S>,
}

impl<S: ProcessedStore> InboxConsumer<S> {
    /// Creates a consumer backed by `store`.
    #[must_use]
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }

    /// Handles `message_id` at most once.
    ///
    /// If the id has not been handled, `handler` runs and the id is recorded as
    /// processed. If the id was already processed, `handler` does not run and
    /// `Ok(false)` is returned.
    ///
    /// # Errors
    ///
    /// Returns [`InboxError`] when the store fails or the handler fails.
    pub async fn handle<F, Fut>(&self, message_id: &str, handler: F) -> Result<bool, InboxError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(), InboxError>> + Send,
    {
        if self.store.is_processed(message_id).await? {
            return Ok(false);
        }
        self.store.mark_processed(message_id).await?;
        handler().await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn first_delivery_runs_handler_and_records_id() {
        let store = Arc::new(InMemoryProcessedStore::new());
        let consumer = InboxConsumer::new(store.clone());
        let mut calls = 0;
        let handled = consumer
            .handle("msg-1", || {
                calls += 1;
                async { Ok(()) }
            })
            .await
            .unwrap();
        assert!(handled);
        assert_eq!(calls, 1);
        assert_eq!(store.len(), 1);
    }

    #[tokio::test]
    async fn duplicate_delivery_is_skipped() {
        let store = Arc::new(InMemoryProcessedStore::new());
        let consumer = InboxConsumer::new(store.clone());
        let mut calls = 0;
        let first = consumer
            .handle("msg-1", || {
                calls += 1;
                async { Ok(()) }
            })
            .await
            .unwrap();
        let second = consumer
            .handle("msg-1", || {
                calls += 1;
                async { Ok(()) }
            })
            .await
            .unwrap();
        assert!(first);
        assert!(!second);
        assert_eq!(calls, 1);
    }

    #[tokio::test]
    async fn distinct_ids_each_run() {
        let store = Arc::new(InMemoryProcessedStore::new());
        let consumer = InboxConsumer::new(store.clone());
        let mut calls = 0;
        for i in 0..3 {
            let handled = consumer
                .handle(&format!("msg-{i}"), || {
                    calls += 1;
                    async { Ok(()) }
                })
                .await
                .unwrap();
            assert!(handled);
        }
        assert_eq!(calls, 3);
        assert_eq!(store.len(), 3);
    }

    #[tokio::test]
    async fn handler_error_is_propagated() {
        let store = Arc::new(InMemoryProcessedStore::new());
        let consumer = InboxConsumer::new(store.clone());
        let result = consumer
            .handle("msg-1", || async { Err(InboxError("boom".into())) })
            .await;
        assert_eq!(result, Err(InboxError("boom".into())));
        assert_eq!(store.len(), 1);
    }
}
