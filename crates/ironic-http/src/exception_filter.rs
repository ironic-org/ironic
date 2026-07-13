//! Exception filter support for the Ironic HTTP layer.

use std::sync::Arc;

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

/// Catches an [`HttpError`] and produces a framework response.
///
/// Implement this trait to handle specific error codes or statuses.
pub trait ExceptionFilter: Send + Sync + 'static {
    /// Handles a caught exception and returns a framework response.
    ///
    /// Return `Ok(response)` to handle the error, or `Err(HttpError)` to
    /// fall through to the next filter or the default error handler.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when the filter itself fails.
    fn catch(
        &self,
        error: &HttpError,
        context: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError>;
}

/// Collects exception filters at a given scope.
#[derive(Clone, Default)]
pub(crate) struct ExceptionFilterSet {
    filters: Vec<Arc<dyn ExceptionFilter>>,
}

impl ExceptionFilterSet {
    pub(crate) const fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.filters.len()
    }

    pub(crate) fn push(&mut self, filter: Arc<dyn ExceptionFilter>) {
        self.filters.push(filter);
    }

    pub(crate) fn append(&mut self, other: &mut Self) {
        self.filters.append(&mut other.filters);
    }

    /// Runs all filters in registration order, returning the first successful
    /// response. If no filter catches the error, returns `None`.
    pub(crate) fn catch(
        &self,
        error: &HttpError,
        context: &FilterContext,
    ) -> Option<Result<FrameworkResponse, HttpError>> {
        for filter in &self.filters {
            let result = filter.catch(error, context);
            if let Ok(ref _resp) = result {
                return Some(result);
            }
            if result.is_err() {
                return Some(result);
            }
        }
        None
    }
}
