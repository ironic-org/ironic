//! Shared identifiers, error primitives, and response helpers for Ironic.

/// Standard error codes and response patterns.
pub mod error_codes;

/// The result type used by framework operations.
pub type AppResult<T> = Result<T, AppError>;

/// A top-level framework failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AppError {
    /// A feature has not been implemented yet.
    #[error("Ironic feature is not implemented: {0}")]
    NotImplemented(&'static str),
}
