//! Converts `sqlx::Error` variants into structured `HttpError` responses.
//!
//! Usage:
//! ```ignore
//! use ironic::http::SqlxErrorExt;
//!
//! let row = sqlx::query("SELECT ...")
//!     .fetch_optional(&pool)
//!     .await
//!     .map_db_err("USER", "FIND")?;
//! ```
//!
//! `RowNotFound` → 404, pool/timeout errors → 503, everything else → 500.

use crate::HttpError;

/// Extension trait that maps `sqlx::Error` to an `HttpError` with descriptive
/// codes based on the entity name and operation.
pub trait SqlxErrorExt {
    /// Maps a `sqlx::Error` to an `HttpError`.
    ///
    /// - `RowNotFound` → `NOT_FOUND` (404)
    /// - `PoolClosed | PoolTimedOut` → `SERVICE_UNAVAILABLE` (503)
    /// - Everything else → `INTERNAL_SERVER_ERROR` (500)
    fn map_db_err(self, entity: &str, operation: &str) -> HttpError;
}

impl SqlxErrorExt for sqlx::Error {
    fn map_db_err(self, entity: &str, operation: &str) -> HttpError {
        match &self {
            sqlx::Error::RowNotFound => {
                HttpError::not_found("DB_ROW_NOT_FOUND", format!("{entity} not found"))
            }
            sqlx::Error::PoolClosed | sqlx::Error::PoolTimedOut => HttpError::new(
                crate::HttpStatus::SERVICE_UNAVAILABLE,
                "DB_UNAVAILABLE",
                format!("Database unavailable during {entity}:{operation}"),
            ),
            _ => HttpError::internal("DB_ERROR", format!("{entity}:{operation} failed: {self}")),
        }
    }
}
