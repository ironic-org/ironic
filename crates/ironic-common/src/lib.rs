#![doc = "Shared identifiers and error primitives for Ironic."]

/// The result type used by framework operations.
pub type FrameworkResult<T> = Result<T, FrameworkError>;

/// A top-level framework failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FrameworkError {
    /// A feature has not been implemented yet.
    #[error("Ironic feature is not implemented: {0}")]
    NotImplemented(&'static str),
}
