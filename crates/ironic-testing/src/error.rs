use rustframe_core::{ApplicationError, HttpApplicationBuildError, ModuleError};
use rustframe_di::ResolveError;

/// A failure while compiling an isolated test module.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TestBuildError {
    /// The module graph is invalid.
    #[error(transparent)]
    Module(#[from] ModuleError),
    /// Provider registration or route compilation failed.
    #[error(transparent)]
    Http(#[from] HttpApplicationBuildError),
    /// An eager provider could not be resolved.
    #[error(transparent)]
    Resolve(#[from] ResolveError),
    /// A complete test application could not initialize.
    #[error(transparent)]
    Application(#[from] ApplicationError),
}
