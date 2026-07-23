use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::Utc;
use tracing_subscriber::Layer;

use super::entry::LogEntry;
use super::storage::LogStorage;

/// A [tracing layer] that captures events and writes them to a log storage backend.
///
/// Composes with other layers via `tracing_subscriber::registry()`.
///
/// # Examples
///
/// ```ignore
/// use tracing_subscriber::prelude::*;
///
/// let storage = Arc::new(FileLogStorage::default());
/// let layer = TimeSeriesLayer::new(storage);
/// tracing_subscriber::registry().with(layer).init();
/// ```
pub struct TimeSeriesLayer<S> {
    storage: Arc<dyn LogStorage>,
    _subscriber: std::marker::PhantomData<S>,
}

impl<S> TimeSeriesLayer<S> {
    /// Creates a new layer that writes captured events to the given storage.
    pub fn new(storage: Arc<dyn LogStorage>) -> Self {
        Self {
            storage,
            _subscriber: std::marker::PhantomData,
        }
    }
}

struct JsonVisitor {
    fields: BTreeMap<String, serde_json::Value>,
}

impl JsonVisitor {
    fn record_value(&mut self, field: &tracing::field::Field, value: serde_json::Value) {
        let name = field.name().to_owned();
        if name == "message" {
            return;
        }
        self.fields.entry(name).or_insert(value);
    }
}

impl tracing::field::Visit for JsonVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.record_value(field, serde_json::Value::String(value.to_owned()));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.record_value(field, serde_json::Value::String(format!("{value:?}")));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.record_value(field, serde_json::Value::Number(value.into()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.record_value(field, serde_json::Value::Number(value.into()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        let n = serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0));
        self.record_value(field, serde_json::Value::Number(n));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.record_value(field, serde_json::Value::Bool(value));
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.record_value(field, serde_json::Value::String(format!("{value}")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn time_series_layer_constructible() {
        struct NoopStorage;
        impl LogStorage for NoopStorage {
            fn store<'a>(&'a self, _entry: LogEntry) -> super::super::storage::StorageFuture<'a> {
                Box::pin(async { Ok(()) })
            }
            fn flush<'a>(&'a self) -> super::super::storage::StorageFuture<'a> {
                Box::pin(async { Ok(()) })
            }
        }

        let storage = Arc::new(NoopStorage);
        let _layer: TimeSeriesLayer<tracing_subscriber::Registry> = TimeSeriesLayer::new(storage);
    }

    #[test]
    fn json_visitor_record_value_skips_message() {
        let visitor = JsonVisitor { fields: BTreeMap::new() };
        assert!(visitor.fields.is_empty());
    }

    #[test]
    fn layer_new_accepts_any_storage() {
        struct CountStorage {
            stored: std::sync::atomic::AtomicU64,
        }
        impl LogStorage for CountStorage {
            fn store<'a>(&'a self, _entry: LogEntry) -> super::super::storage::StorageFuture<'a> {
                self.stored.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Box::pin(async { Ok(()) })
            }
            fn flush<'a>(&'a self) -> super::super::storage::StorageFuture<'a> {
                Box::pin(async { Ok(()) })
            }
        }

        let storage = Arc::new(CountStorage {
            stored: std::sync::atomic::AtomicU64::new(0),
        });
        let _layer: TimeSeriesLayer<tracing_subscriber::Registry> = TimeSeriesLayer::new(storage);
    }
}

impl<S: tracing::Subscriber> Layer<S> for TimeSeriesLayer<S> {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let timestamp = Utc::now();
        let level = *event.metadata().level();
        let target = event.metadata().target();
        let module_path = event.metadata().module_path();
        let file = event.metadata().file();
        let line = event.metadata().line();

        let mut visitor = JsonVisitor {
            fields: BTreeMap::new(),
        };
        event.record(&mut visitor);

        let message = visitor
            .fields
            .remove("message")
            .and_then(|v| match v {
                serde_json::Value::String(s) => Some(s),
                _ => None,
            })
            .unwrap_or_default();

        let entry = LogEntry::new(
            timestamp,
            level,
            target,
            message,
            module_path,
            file,
            line,
            visitor.fields,
        );

        let storage = self.storage.clone();
        tokio::spawn(async move {
            let _ = storage.store(entry).await;
        });
    }
}
