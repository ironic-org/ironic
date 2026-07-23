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
    ///
    /// # Errors
    ///
    /// Maps the contained `sqlx::Error` into an `HttpError`. See
    /// [`SqlxErrorExt::map_db_err`] for the error mapping rules.
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

#[cfg(all(test, feature = "sqlx"))]
mod tests {
    use super::*;

    #[test]
    fn row_not_found_maps_to_404() {
        let err = sqlx::Error::RowNotFound;
        let http_err = err.map_db_err("User", "find_by_id");
        assert_eq!(http_err.status(), crate::HttpStatus::NOT_FOUND);
        assert_eq!(http_err.code(), "DB_ROW_NOT_FOUND");
    }

    #[test]
    fn pool_closed_maps_to_503() {
        let err = sqlx::Error::PoolClosed;
        let http_err = err.map_db_err("Db", "query");
        assert_eq!(http_err.status(), crate::HttpStatus::SERVICE_UNAVAILABLE);
        assert_eq!(http_err.code(), "DB_UNAVAILABLE");
    }

    #[test]
    fn pool_timed_out_maps_to_503() {
        let err = sqlx::Error::PoolTimedOut;
        let http_err = err.map_db_err("Db", "query");
        assert_eq!(http_err.status(), crate::HttpStatus::SERVICE_UNAVAILABLE);
        assert_eq!(http_err.code(), "DB_UNAVAILABLE");
    }

    #[test]
    fn generic_error_maps_to_500() {
        let err = sqlx::Error::Protocol(String::from("protocol violation"));
        let http_err = err.map_db_err("Order", "create");
        assert_eq!(http_err.status(), crate::HttpStatus::INTERNAL_SERVER_ERROR);
        assert_eq!(http_err.code(), "DB_ERROR");
        assert!(http_err.message().contains("Order:create"));
    }

    #[test]
    fn sqlx_result_ext_ok_passes_through() {
        let result: Result<i32, sqlx::Error> = Ok(42);
        let mapped = result.map_db_err("Entity", "op");
        assert_eq!(mapped.unwrap(), 42);
    }

    #[test]
    fn sqlx_result_ext_err_maps() {
        let result: Result<i32, sqlx::Error> = Err(sqlx::Error::RowNotFound);
        let mapped = result.map_db_err("Product", "get");
        let err = mapped.unwrap_err();
        assert_eq!(err.code(), "DB_ROW_NOT_FOUND");
        assert!(err.message().contains("Product"));
    }
}
