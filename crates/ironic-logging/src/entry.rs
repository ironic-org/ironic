use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::Level;

/// A single logging log entry captured from a tracing event.
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
