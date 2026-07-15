//! Structured logging for Ironic applications.
//!
//! Captures all `tracing` events and writes them as JSON Lines (`.jsonl`)
//! files organised by day under a `.logs/` directory. The storage backend is
//! pluggable via the [`LogStorage`] trait, allowing logs to be routed to
//! databases, object stores, or any other sink.
//!
//! # Feature flag
//!
//! Enable `logging` in your `Cargo.toml`:
//!
//! ```toml
//! ironic = { features = ["logging"] }
//! ```
//!
//! # Quick start
//!
//! The [`TimeSeriesModule`] automatically initialises the logging layer when
//! registered with your application:
//!
//! ```ignore
//! use ironic::prelude::*;
//!
//! #[derive(Module)]
//! #[module(imports = [TimeSeriesModule, ...])]
//! struct AppModule;
//! ```
//!
//! All `tracing::info!()`, `tracing::warn!()` etc. calls are then captured
//! and written to `.logs/YYYY-MM-DD.jsonl`.
//!
//! # Custom storage backend
//!
//! Implement [`LogStorage`] for your database:
//!
//! ```ignore
//! use ironic::logging::{LogStorage, LogEntry, StorageError, StorageFuture};
//!
//! struct PgLogStorage { pool: sqlx::PgPool }
//!
//! impl LogStorage for PgLogStorage {
//!     fn store<'a>(&'a self, entry: LogEntry) -> StorageFuture<'a> {
//!         Box::pin(async move {
//!             sqlx::query("INSERT INTO logs ...").execute(&self.pool).await?;
//!             Ok(())
//!         })
//!     }
//!     fn flush<'a>(&'a self) -> StorageFuture<'a> { Box::pin(async { Ok(()) }) }
//! }
//! ```
//!
//! Then pass it to [`TimeSeriesConfig`]:
//!
//! ```ignore
//! TimeSeriesConfig::default().with_storage(Arc::new(PgLogStorage { pool }));
//! ```

mod entry;
mod layer;
mod storage;

pub use entry::LogEntry;
pub use layer::TimeSeriesLayer;
pub use storage::{FileLogStorage, LogStorage, StorageError, StorageFuture};

use std::sync::Arc;

use tracing_subscriber::prelude::*;

/// Configuration for the logging logging system.
pub struct TimeSeriesConfig {
    /// Backend that persists log entries.
    /// Defaults to [`FileLogStorage`] writing to `.logs/`.
    pub storage: Arc<dyn LogStorage>,
}

impl std::fmt::Debug for TimeSeriesConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimeSeriesConfig")
            .field("storage", &"Arc<dyn LogStorage>")
            .finish()
    }
}

impl Clone for TimeSeriesConfig {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
        }
    }
}

impl Default for TimeSeriesConfig {
    fn default() -> Self {
        Self {
            storage: Arc::new(FileLogStorage::logs_dir()),
        }
    }
}

impl TimeSeriesConfig {
    /// Override the default storage backend.
    #[must_use]
    pub fn with_storage(mut self, storage: Arc<dyn LogStorage>) -> Self {
        self.storage = storage;
        self
    }

    /// Initialises the global tracing subscriber with the logging layer.
    ///
    /// # Panics
    ///
    /// Panics if a global subscriber has already been set.
    pub fn init(self) {
        let layer = TimeSeriesLayer::new(self.storage);
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    tracing_subscriber::filter::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")),
                ),
            )
            .with(layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("tracing subscriber must be set once");
    }

    /// Returns the logging layer for manual composition with other layers.
    pub fn layer(self) -> TimeSeriesLayer<tracing_subscriber::registry::Registry> {
        TimeSeriesLayer::<tracing_subscriber::registry::Registry>::new(self.storage)
    }
}

/// Ironic module that registers the logging logging layer.
///
/// Import it into your application module:
///
/// ```ignore
/// #[derive(Module)]
/// #[module(imports = [TimeSeriesModule])]
/// struct AppModule;
/// ```
pub struct TimeSeriesModule;

impl ironic_core::Module for TimeSeriesModule {
    fn definition() -> ironic_core::ModuleDefinition {
        // Initialise the logging layer with default config.
        // The layer is set once; subsequent calls are no-ops.
        let config = TimeSeriesConfig::default();

        let layer = TimeSeriesLayer::new(config.storage);
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    tracing_subscriber::filter::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")),
                ),
            )
            .with(layer);

        let _ = tracing::subscriber::set_global_default(subscriber);

        ironic_core::ModuleDefinition::builder::<Self>().build()
    }
}

/// Ironic-specific logging macros.
///
/// These are thin wrappers around `tracing` macros, provided for convenience.
/// The logging layer captures all `tracing` events automatically, so you
/// can also use `tracing::info!()` directly.
pub mod log {
    #[doc(hidden)]
    pub use tracing::{debug, error, info, trace, warn};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_file_storage() {
        let config = TimeSeriesConfig::default();
        // FileLogStorage is not Debug-printable in detail, but we verify it compiles.
        let _ = config;
    }

    #[test]
    fn layer_can_be_constructed() {
        let storage = Arc::new(FileLogStorage::default());
        let _layer: TimeSeriesLayer<tracing_subscriber::Registry> = TimeSeriesLayer::new(storage);
    }

    #[tokio::test]
    async fn log_entry_roundtrip() {
        use std::collections::BTreeMap;
        let entry = LogEntry::new(
            chrono::Utc::now(),
            tracing::Level::ERROR,
            "my_app::service",
            "something broke".into(),
            Some("my_app::service"),
            Some("src/service.rs"),
            Some(42),
            BTreeMap::from([("user_id".into(), serde_json::json!("abc-123"))]),
        );
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("ERROR"));
        assert!(json.contains("something broke"));
        assert!(json.contains("abc-123"));
    }
}
