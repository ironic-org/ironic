//! Converts `sqlx::Error` variants into structured `HttpError` responses.
//!
//! Usage:
//! ```ignore
//! use ironic::SqlxErrorExt;
//!
//! let row = sqlx::query("SELECT ...")
//!     .fetch_optional(&pool)
//!     .await
//!     .map_db_err("USER", "FIND")?;
//! ```
//!
//! `RowNotFound` → 404, pool/timeout errors → 503, everything else → 500.

use crate::HttpError;

/// Extension trait that maps sqlx errors to `HttpError`.
pub trait SqlxErrorExt {
    /// Maps a `sqlx::Error` to an `HttpError`.
    fn map_db_err(self, entity: &str, operation: &str) -> HttpError;
}

/// Extension trait that maps `Result<T, sqlx::Error>` to `Result<T, HttpError>`.
pub trait SqlxResultExt<T> {
    /// Maps the error of a `Result` using [`SqlxErrorExt::map_db_err`].
    fn map_db_err(self, entity: &str, operation: &str) -> Result<T, HttpError>;
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

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn map_db_err(self, entity: &str, operation: &str) -> Result<T, HttpError> {
        self.map_err(|e| e.map_db_err(entity, operation))
    }
}
