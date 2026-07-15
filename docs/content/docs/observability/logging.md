---
title: Structured Logging
description: Capture structured time-series logs, write to JSON Lines files, and route to any database with a pluggable storage backend.
---

# Structured Logging

## What you'll learn

- Enable structured logging with the `logging` feature
- Use `TimeSeriesModule` to automatically capture all `tracing` events
- Write structured logs to `.logs/YYYY-MM-DD.jsonl` with daily rotation
- Use `ironic::log::{info, warn, error, debug, trace}` macros
- Understand `LogEntry` fields and the JSON Lines format
- Implement a custom `LogStorage` backend for databases
- Compose `TimeSeriesLayer` with other tracing layers (OpenTelemetry, etc.)
- Configure the storage directory and write behaviour
- Test logging with both in-memory and file-backed storage

---

## Enabling structured logging

```toml
ironic = { features = ["logging"] }
```

The `logging` feature is included in Ironic's default features, so new projects
created with `ironic new` already have it enabled.

---

## Quick start

Register `TimeSeriesModule` in your application module:

```rust
use ironic::prelude::*;

#[derive(Module)]
#[module(imports = [TimeSeriesModule])]
struct AppModule;
```

All `tracing::info!()`, `tracing::warn!()`, `tracing::error!()`, and similar
calls are now automatically captured and written to `.logs/YYYY-MM-DD.jsonl` in
JSON Lines format — one JSON object per line, one file per day.

### Using ironic::log macros

For convenience, Ironic re-exports the standard tracing macros under
`ironic::log`:

```rust
use ironic::log::{info, warn, error, debug, trace};

// Basic message
info!("server started on port 8080");

// With structured fields
warn!(disk_usage = %"85%", "disk nearing capacity");

// With typed fields
error!(user_id = 42, action = "purchase", "database query failed");

// Debug-format a value
let payload = vec!["a", "b"];
debug!(payload = ?payload, "processing batch");
```

These are thin wrappers around `tracing` macros — the logging layer captures
events regardless of which path you use.

### Using tracing directly (same result)

```rust
use tracing::{info, warn, error, debug, trace};

info!("server started on port 8080");
warn!(disk_usage = %"85%", "disk nearing capacity");
error!(user_id = 42, "database query failed");
```

Both approaches produce identical JSON Lines output in `.logs/YYYY-MM-DD.jsonl`.
The `TimeSeriesLayer` captures all `tracing` events automatically — you just
write logs, and the file storage happens behind the scenes.

---

## Architecture

The logging system has three layers:

```
┌──────────────────────────────────────────────┐
│              Application code                │
│  tracing::info!("...") / ironic::log::info!   │
└──────────────────┬───────────────────────────┘
                   │  event
┌──────────────────▼───────────────────────────┐
│         TimeSeriesLayer (tracing subscriber) │
│  ┌────────────────────────────────────────┐  │
│  │  on_event() → LogEntry → tokio::spawn  │  │
│  │  (async write to storage backend)       │  │
│  └────────────────────────────────────────┘  │
└──────────────────┬───────────────────────────┘
                   │  store()
┌──────────────────▼───────────────────────────┐
│          LogStorage (pluggable backend)       │
│  ┌────────────────────────────────────────┐  │
│  │  FileLogStorage → .logs/*.jsonl        │  │
│  │  PgLogStorage   → PostgreSQL table     │  │
│  │  YourBackend    → any sink you want    │  │
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

The `TimeSeriesLayer` implements `tracing_subscriber::Layer` and is composed
into the global subscriber. Every `tracing` event triggers `on_event()`, which
constructs a `LogEntry` and sends it to the storage backend via `tokio::spawn`
— so logging never blocks the request handler.

---

## Log file format

### Directory structure

```
.logs/
├── 2026-07-16.jsonl
├── 2026-07-17.jsonl
└── 2026-07-18.jsonl
```

Each day gets its own file. Files are named by UTC date. The `.logs/` directory
is created automatically on first write.

### JSON Lines schema

Every line is a JSON object:

```json
{
  "timestamp": "2026-07-16T10:30:00.123456Z",
  "level": "INFO",
  "target": "my_app::routes",
  "message": "handling hello request",
  "module_path": "my_app::routes",
  "file": "src/routes.rs",
  "line": 12,
  "fields": {
    "user": "alice"
  }
}
```

### Field reference

| Field | Type | Always present | Description |
|-------|------|----------------|-------------|
| `timestamp` | `string` (ISO-8601) | Yes | UTC timestamp with microsecond precision via `chrono::Utc::now()` |
| `level` | `string` | Yes | One of `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` |
| `target` | `string` | Yes | Module target from `tracing` metadata |
| `message` | `string` | Yes | The log message (from the fmt string or `message` field) |
| `module_path` | `string` | No | Fully-qualified Rust module path |
| `file` | `string` | No | Source file name |
| `line` | `number` | No | Source line number |
| `fields` | `object` | No | Structured key-value pairs from the event's fields |

The `fields` object can contain any JSON value — strings, numbers, booleans,
nested objects, or arrays:

```rust
info!(
    user_id = 42,
    action = "purchase",
    amount = 29.99,
    items = ?["widget", "gadget"],  // debug-format as a string
);
// fields: { "user_id": 42, "action": "purchase", "amount": 29.99, "items": "[\"widget\", \"gadget\"]" }
```

---

## TimeSeriesConfig

```rust
use std::sync::Arc;
use ironic::logging::{TimeSeriesConfig, FileLogStorage};

// Default: writes to .logs/
let config = TimeSeriesConfig::default();
config.init();
```

### Methods

| Method | Description |
|--------|-------------|
| `with_storage(storage)` | Replace the default `FileLogStorage` with a custom backend |
| `init()` | Set the composed subscriber as the global default (panics if already set) |
| `layer()` | Return the `TimeSeriesLayer` for manual composition with other layers |

### Custom storage directory

```rust
TimeSeriesConfig::default()
    .with_storage(Arc::new(
        FileLogStorage::new("/var/log/my-app"),
    ))
    .init();
```

### Storage error behaviour

When the storage backend fails (e.g. disk full, database unavailable), the
error is silently swallowed — the request handler is never affected. Errors
are visible in the application's stderr output if the default `tracing::fmt`
layer is also registered (which `TimeSeriesModule` does automatically).

---

## FileLogStorage

```rust
use ironic::logging::FileLogStorage;

// Default path: .logs/
let storage = FileLogStorage::logs_dir();

// Custom path:
let storage = FileLogStorage::new("/var/log/my-app");
```

### Constructor reference

| Constructor | Behaviour |
|-------------|-----------|
| `FileLogStorage::new(path)` | Writes `.jsonl` files into `path`. Creates the directory on first write. |
| `FileLogStorage::logs_dir()` | Shorthand for `FileLogStorage::new(".logs")` |
| `FileLogStorage::default()` | Same as `logs_dir()` (via `Default` trait) |

### Write behaviour

- Files are opened with `OpenOptions::create(true).append(true)` — existing
  daily files are never truncated.
- Every write is followed by an immediate `flush()` on the file handle to
  minimise data loss on crash.
- A `Mutex` guards the writer state so the storage can be shared across
  threads. Contention is extremely low because writes happen on a
  `tokio::spawn` task.

### Thread safety

`FileLogStorage` is `Send + Sync` and can be shared across Tokio tasks. The
internal mutex is held only for the duration of a single `write_line()` call
(JSON serialization + disk write).

---

## Pluggable storage backend

Implement the `LogStorage` trait to route logs to any sink:

### LogStorage trait

```rust
/// Future type returned by LogStorage methods.
pub type StorageFuture<'a> = Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + 'a>>;

pub trait LogStorage: Send + Sync {
    /// Persist a single log entry.
    fn store<'a>(&'a self, entry: LogEntry) -> StorageFuture<'a>;
    /// Flush any pending writes (called on shutdown).
    fn flush<'a>(&'a self) -> StorageFuture<'a>;
}
```

### StorageError

```rust
pub enum StorageError {
    /// An I/O operation failed.
    Io(std::io::Error),
    /// JSON serialization failed.
    Serialization(serde_json::Error),
    /// Backend-specific error.
    Backend(String),
}
```

### PostgreSQL backend example

```rust
use std::sync::Arc;
use ironic::logging::{LogEntry, LogStorage, StorageError, StorageFuture};

struct PgLogStorage {
    pool: sqlx::PgPool,
}

impl LogStorage for PgLogStorage {
    fn store<'a>(&'a self, entry: LogEntry) -> StorageFuture<'a> {
        Box::pin(async move {
            sqlx::query(
                r#"
                INSERT INTO logs (timestamp, level, target, message, fields)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(entry.timestamp)
            .bind(&entry.level)
            .bind(&entry.target)
            .bind(&entry.message)
            .bind(serde_json::to_value(&entry.fields).unwrap_or_default())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::Backend(e.to_string()))?;

            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> StorageFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}

// Usage
let pool = PgPoolOptions::new()
    .connect("postgres://localhost/my_app")
    .await?;

let config = TimeSeriesConfig::default()
    .with_storage(Arc::new(PgLogStorage { pool }));
config.init();
```

### ClickHouse backend example

```rust
use ironic::logging::{LogEntry, LogStorage, StorageError, StorageFuture};

struct ClickHouseStorage {
    client: clickhouse::Client,
}

impl LogStorage for ClickHouseStorage {
    fn store<'a>(&'a self, entry: LogEntry) -> StorageFuture<'a> {
        Box::pin(async move {
            let mut inserter = self.client
                .inserter::<serde_json::Value>("logs")?
                .write(&serde_json::to_value(&entry)?)
                .map_err(|e| StorageError::Backend(e.to_string()))?;
            inserter.commit().await?;
            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> StorageFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}
```

---

## Composing with other layers

Use `TimeSeriesConfig::layer()` to get the raw `TimeSeriesLayer` for manual
composition with `tracing_subscriber::registry()`:

```rust
use tracing_subscriber::prelude::*;
use ironic::logging::TimeSeriesConfig;

let subscriber = tracing_subscriber::registry()
    .with(
        tracing_subscriber::fmt::layer()
            .with_filter(
                tracing_subscriber::filter::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")),
            ),
    )
    .with(TimeSeriesConfig::default().layer());

tracing::subscriber::set_global_default(subscriber)
    .expect("tracing subscriber must be set once");
```

### With OpenTelemetry

```rust
use tracing_subscriber::prelude::*;
use ironic::logging::TimeSeriesConfig;
use ironic::telemetry::{TelemetryConfig, init_tracing};
```

The `TimeSeriesModule` provided by the `logging` feature and the
`init_tracing` from the `telemetry` feature both call
`tracing::subscriber::set_global_default()` — only the first one succeeds.
If you use both features, compose the layers manually:

```rust
use tracing_subscriber::prelude::*;
use ironic::telemetry::TelemetryConfig;
use ironic::logging::TimeSeriesConfig;

let subscriber = tracing_subscriber::registry()
    .with(
        tracing_subscriber::fmt::layer()
            .with_filter(
                tracing_subscriber::filter::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")),
            ),
    )
    .with(
        opentelemetry::global::tracer_provider()
            .tracer("my_app")
            .pipe(tracing_opentelemetry::layer),
    )
    .with(TimeSeriesConfig::default().layer());

tracing::subscriber::set_global_default(subscriber)
    .expect("tracing subscriber must be set once");
```

---

## TimeSeriesLayer

The `TimeSeriesLayer` is the core component that bridges `tracing` events to
the storage backend.

```rust
use std::sync::Arc;
use ironic::logging::{TimeSeriesLayer, FileLogStorage};

let storage = Arc::new(FileLogStorage::logs_dir());
let layer: TimeSeriesLayer<tracing_subscriber::Registry> =
    TimeSeriesLayer::new(storage);
```

### How it captures events

1. `on_event()` is called for every `tracing` event
2. A `JsonVisitor` walks the event's structured fields using the
   `tracing::field::Visit` trait
3. A `LogEntry` is constructed with the timestamp, level, target,
   message, source location, and structured fields
4. The entry is sent to the storage backend via `tokio::spawn`

### Performance characteristics

- **No blocking calls** in the event handler — the entry is constructed
  synchronously (microseconds) and dispatched to a Tokio task
- **Async storage writes** happen off the critical path
- **`tokio::spawn` overhead** is negligible — roughly 1–2 µs per event
- **Backpressure**: if the storage backend is slow, entries queue in
  Tokio's task scheduler. Under extreme load, consider a bounded channel
  between the layer and the storage writer.

---

## LogEntry

```rust
use chrono::{DateTime, Utc};

pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: BTreeMap<String, serde_json::Value>,
}
```

All fields are `pub` and the struct derives `Clone`, `Debug`, and
`Serialize` — you can inspect, transform, or forward entries in custom
storage backends.

---

## StorageFuture

```rust
pub type StorageFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + 'a>>;
```

Custom storage backends return a `StorageFuture` from both `store()` and
`flush()`. Use `Box::pin(async move { ... })` to construct one.

---

## Testing

### Testing with FileLogStorage (temporary directory)

```rust
#[cfg(test)]
mod tests {
    use ironic::logging::{FileLogStorage, LogEntry, LogStorage};
    use std::collections::BTreeMap;
    use chrono::Utc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn logs_are_written_to_daily_file() {
        let dir = TempDir::new().unwrap();
        let storage = FileLogStorage::new(dir.path());

        let entry = LogEntry::new(
            Utc::now(),
            &tracing::Level::INFO,
            "test",
            "hello".into(),
            None,
            None,
            None,
            BTreeMap::new(),
        );
        storage.store(entry).await.unwrap();
        storage.flush().await.unwrap();

        let date = Utc::now().format("%Y-%m-%d").to_string();
        let path = dir.path().join(format!("{date}.jsonl"));
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("\"hello\""));
    }

    #[tokio::test]
    async fn multiple_entries_are_appended() {
        let dir = TempDir::new().unwrap();
        let storage = FileLogStorage::new(dir.path());

        for i in 0..5 {
            let entry = LogEntry::new(
                Utc::now(),
                &tracing::Level::INFO,
                "test",
                format!("msg {i}"),
                None,
                None,
                None,
                BTreeMap::new(),
            );
            storage.store(entry).await.unwrap();
        }
        storage.flush().await.unwrap();

        let date = Utc::now().format("%Y-%m-%d").to_string();
        let path = dir.path().join(format!("{date}.jsonl"));
        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content.lines().count(), 5);
    }
}
```

### Testing with a mock backend

```rust
use std::sync::Mutex;
use ironic::logging::{LogEntry, LogStorage, StorageError, StorageFuture};

struct MockStorage {
    entries: Mutex<Vec<LogEntry>>,
}

impl LogStorage for MockStorage {
    fn store<'a>(&'a self, entry: LogEntry) -> StorageFuture<'a> {
        Box::pin(async move {
            self.entries.lock().unwrap().push(entry);
            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> StorageFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}

#[tokio::test]
async fn mock_storage_captures_entries() {
    let storage = MockStorage {
        entries: Mutex::new(Vec::new()),
    };

    let entry = LogEntry::new(
        Utc::now(),
        &tracing::Level::WARN,
        "test",
        "warning message".into(),
        None,
        None,
        None,
        BTreeMap::new(),
    );
    storage.store(entry).await.unwrap();

    assert_eq!(storage.entries.lock().unwrap().len(), 1);
}
```

---

## Common mistakes

| Mistake | Fix |
|---------|------|
| Missing `logging` feature | Add `ironic = { features = ["logging"] }` to `Cargo.toml` |
| No `TimeSeriesModule` import | Add `TimeSeriesModule` to your module's `imports` array |
| Two layers call `set_global_default` | Use `TimeSeriesConfig::layer()` for manual composition instead of `init()` |
| Logs not appearing in `.logs/` | Check file permissions; the directory is created on first write, not at startup |
| Expecting real-time flush | Writes are async via `tokio::spawn`. There is a ~1ms delay between event and file write |
| Database backend not flushing | Call `storage.flush()` periodically or implement auto-flush |
| Using `init()` after `set_global_default` | `init()` panics if a subscriber is already set. Use `layer()` for composition |
| Huge log volume slowing the app | Use a bounded channel between layer and storage, or batch entries before writing |

---

## What you learned

- [x] `TimeSeriesModule` automatically captures all `tracing` events without code changes
- [x] Logs are written to `.logs/YYYY-MM-DD.jsonl` — one JSON object per line, one file per day
- [x] `ironic::log::{info, warn, error, debug, trace}` re-exports are available for convenience
- [x] `FileLogStorage` writes to daily files with automatic directory creation and immediate flush
- [x] `LogStorage` trait enables pluggable backends for PostgreSQL, ClickHouse, S3, etc.
- [x] `TimeSeriesConfig::layer()` composes with other tracing layers (OpenTelemetry, etc.)
- [x] The `TimeSeriesLayer` runs entry construction synchronously and dispatches storage to `tokio::spawn`
- [x] Mock and temporary-directory patterns make logging testable
