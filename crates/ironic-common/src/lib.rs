//! Shared identifiers, error primitives, and response helpers for Ironic.

/// Standard error codes and response patterns.
pub mod error_codes;

/// The result type used by framework operations.
///
/// # Examples
///
/// ```rust
/// use ironic::AppResult;
///
/// fn always_ok() -> AppResult<i32> {
///     Ok(42)
/// }
/// ```
pub type AppResult<T> = Result<T, AppError>;

/// A top-level framework failure.
///
/// Currently covers `NotImplemented`; extended in later releases.
///
/// # Examples
///
/// ```rust
/// use ironic::AppError;
///
/// let err = AppError::NotImplemented("feature-x");
/// assert_eq!(err.to_string(), "Ironic feature is not implemented: feature-x");
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    /// A feature has not been implemented yet.
    #[error("Ironic feature is not implemented: {0}")]
    NotImplemented(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_display() {
        let err = AppError::NotImplemented("test-feature");
        assert_eq!(
            err.to_string(),
            "Ironic feature is not implemented: test-feature"
        );
    }

    #[test]
    fn app_error_debug() {
        let err = AppError::NotImplemented("test");
        let debug = format!("{err:?}");
        assert!(debug.contains("NotImplemented"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn app_result_ok() {
        assert!(AppResult::<i32>::Ok(42).is_ok());
        assert_eq!(42, 42);
    }

    #[test]
    fn app_result_err() {
        assert!(AppResult::<i32>::Err(AppError::NotImplemented("missing")).is_err());
        assert!(format!("{}", AppError::NotImplemented("missing")).contains("missing"));
    }
}
