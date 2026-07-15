use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Mutex;

use super::entry::LogEntry;

/// Future type returned by [`LogStorage`] methods.
pub type StorageFuture<'a> = Pin<Box<dyn Future<Output = Result<(), StorageError>> + Send + 'a>>;

/// Error returned by [`LogStorage`] operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// An I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Backend-specific error.
    #[error("Storage backend error: {0}")]
    Backend(String),
}

/// Pluggable storage backend for logging log entries.
///
/// Implementations write entries to files, databases, or any other sink.
/// All methods are `Send + Sync` so backends can be shared across tasks.
pub trait LogStorage: Send + Sync {
    /// Persist a single log entry.
    fn store<'a>(&'a self, entry: LogEntry) -> StorageFuture<'a>;
    /// Flush any pending writes.
    fn flush<'a>(&'a self) -> StorageFuture<'a>;
}

/// Writes logging log entries to `.logs/` as JSON Lines files.
///
/// Each day gets its own file: `.logs/2026-07-16.jsonl`
#[derive(Debug)]
pub struct FileLogStorage {
    directory: PathBuf,
    writer: Mutex<Option<JsonLinesWriter>>,
}

#[derive(Debug)]
struct JsonLinesWriter {
    file: std::fs::File,
    date: String,
}

impl FileLogStorage {
    /// Creates a storage backend rooted at the given directory.
    ///
    /// The directory is created on first write if it doesn't exist.
    pub fn new<P: AsRef<Path>>(directory: P) -> Self {
        Self {
            directory: directory.as_ref().to_owned(),
            writer: Mutex::new(None),
        }
    }

    /// Creates a storage backend rooted at `.logs/` relative to the current directory.
    pub fn logs_dir() -> Self {
        Self::new(".logs")
    }

    fn write_line(&self, date: &str, line: &str) -> Result<(), StorageError> {
        use std::io::Write;

        let mut guard = self.writer.lock().map_err(|e| {
            StorageError::Backend(format!("writer lock poisoned: {e}"))
        })?;

        if guard.as_ref().is_none_or(|w| w.date != date) {
            std::fs::create_dir_all(&self.directory)?;
            let path = self.directory.join(format!("{date}.jsonl"));
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            *guard = Some(JsonLinesWriter {
                file,
                date: date.to_owned(),
            });
        }

        if let Some(ref mut writer) = *guard {
            writeln!(writer.file, "{line}")?;
            writer.file.flush()?;
        }

        Ok(())
    }
}

impl Default for FileLogStorage {
    fn default() -> Self {
        Self::logs_dir()
    }
}

impl LogStorage for FileLogStorage {
    fn store<'a>(&'a self, entry: LogEntry) -> StorageFuture<'a> {
        Box::pin(async move {
            let date = entry.timestamp.format("%Y-%m-%d").to_string();
            let line = serde_json::to_string(&entry)?;
            self.write_line(&date, &line)
        })
    }

    fn flush<'a>(&'a self) -> StorageFuture<'a> {
        Box::pin(async move {
            #[allow(clippy::collapsible_if)]
            if let Ok(mut guard) = self.writer.lock() {
                if let Some(writer) = guard.as_mut() {
                    use std::io::Write;
                    let _ = writer.file.flush();
                }
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::entry::LogEntry;
    use chrono::Utc;
    use std::collections::BTreeMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn writes_jsonl_file() {
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
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"hello\""));
    }

    #[tokio::test]
    async fn appends_to_existing_file() {
        let dir = TempDir::new().unwrap();
        let storage = FileLogStorage::new(dir.path());

        let e1 = LogEntry::new(
            Utc::now(),
            &tracing::Level::INFO,
            "t1",
            "msg1".into(),
            None,
            None,
            None,
            BTreeMap::new(),
        );
        let e2 = LogEntry::new(
            Utc::now(),
            &tracing::Level::WARN,
            "t2",
            "msg2".into(),
            None,
            None,
            None,
            BTreeMap::new(),
        );
        storage.store(e1).await.unwrap();
        storage.store(e2).await.unwrap();
        storage.flush().await.unwrap();

        let date = Utc::now().format("%Y-%m-%d").to_string();
        let path = dir.path().join(format!("{date}.jsonl"));
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content.lines().count(), 2);
    }
}
