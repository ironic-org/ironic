use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::Level;

/// A single logging log entry captured from a tracing event.
///
/// Each entry carries an ISO-8601 timestamp, a log level, message, optional
/// source-location metadata, and structured key-value fields.
///
/// Serialised as JSON when persisted by [`super::storage::LogStorage`].
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    /// ISO-8601 timestamp (UTC).
    pub timestamp: DateTime<Utc>,
    /// Log level.
    pub level: String,
    /// Target module (e.g. "`my_app::service`").
    pub target: String,
    /// The log message.
    pub message: String,
    /// Source module path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_path: Option<String>,
    /// Source file name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Source line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Structured key-value fields.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, serde_json::Value>,
}

impl LogEntry {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        timestamp: DateTime<Utc>,
        level: Level,
        target: &str,
        message: String,
        module_path: Option<&str>,
        file: Option<&str>,
        line: Option<u32>,
        fields: BTreeMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            timestamp,
            level: level_to_str(level),
            target: target.to_owned(),
            message,
            module_path: module_path.map(String::from),
            file: file.map(String::from),
            line,
            fields,
        }
    }
}

fn level_to_str(level: Level) -> String {
    match level {
        Level::TRACE => "TRACE".into(),
        Level::DEBUG => "DEBUG".into(),
        Level::INFO => "INFO".into(),
        Level::WARN => "WARN".into(),
        Level::ERROR => "ERROR".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn level_conversion() {
        assert_eq!(level_to_str(Level::TRACE), "TRACE");
        assert_eq!(level_to_str(Level::DEBUG), "DEBUG");
        assert_eq!(level_to_str(Level::INFO), "INFO");
        assert_eq!(level_to_str(Level::WARN), "WARN");
        assert_eq!(level_to_str(Level::ERROR), "ERROR");
    }

    #[test]
    fn log_entry_creation_and_fields() {
        let mut fields = BTreeMap::new();
        fields.insert("key".into(), serde_json::json!("val"));
        let entry = LogEntry::new(
            Utc::now(),
            Level::INFO,
            "test",
            "hello".into(),
            Some("test_mod"),
            Some("test.rs"),
            Some(10),
            fields,
        );
        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "hello");
        assert_eq!(entry.module_path.as_deref(), Some("test_mod"));
        assert_eq!(entry.file.as_deref(), Some("test.rs"));
        assert_eq!(entry.line, Some(10));
        assert_eq!(
            entry.fields.get("key"),
            Some(&serde_json::json!("val"))
        );
    }

    #[test]
    fn log_entry_serialisation() {
        let entry = LogEntry::new(
            Utc::now(),
            Level::WARN,
            "warn_mod",
            "warning message".into(),
            None,
            None,
            None,
            BTreeMap::new(),
        );
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"WARN\""));
        assert!(json.contains("\"warning message\""));
    }

    #[test]
    fn log_entry_without_fields_omits_fields() {
        let entry = LogEntry::new(
            Utc::now(),
            Level::ERROR,
            "err_mod",
            "error".into(),
            None,
            None,
            None,
            BTreeMap::new(),
        );
        let json = serde_json::to_string(&entry).unwrap();
        // When fields is empty it should be skipped
        assert!(!json.contains("\"fields\""));
    }
}
