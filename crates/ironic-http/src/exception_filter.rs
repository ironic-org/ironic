//! Exception filter support for the Ironic HTTP layer.

use crate::{FrameworkResponse, HttpError, RouteMetadata};

/// Request context available to exception filters.
#[derive(Clone, Debug)]
pub struct FilterContext {
    route_metadata: RouteMetadata,
}

impl FilterContext {
    /// Creates a new filter context.
    #[must_use]
    pub fn new(route_metadata: RouteMetadata) -> Self {
        Self { route_metadata }
    }

    /// Returns typed metadata from the route that caused the exception.
    #[must_use]
    pub fn route_metadata(&self) -> &RouteMetadata {
        &self.route_metadata
    }
}

/// Catches a specific exception type and produces a framework response.
///
/// Implement this trait to handle custom error types.
pub trait ExceptionFilter<E>: Send + Sync + 'static {
    /// Handles a caught exception and returns a framework response.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when the filter itself fails.
    fn catch(
        &self,
        exception: E,
        context: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError>;
}
