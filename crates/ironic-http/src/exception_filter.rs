//! Exception filter support for the Ironic HTTP layer.

use std::sync::Arc;

use crate::{HttpError, Response, RouteMetadata};

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
    fn catch(&self, error: &HttpError, context: &FilterContext) -> Result<Response, HttpError>;
}

/// Extension trait for `Result<T, HttpError>` providing inline exception handling.
pub trait ExceptionExt<T> {
    /// Transforms the error using a closure.
    ///
    /// # Errors
    ///
    /// Returns the error produced by the closure.
    ///
    /// ```ignore
    /// self.auth.login(&dto.username, &dto.password)
    ///     .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
    /// ```
    fn exception<F>(self, f: F) -> Result<T, HttpError>
    where
        F: FnOnce(HttpError) -> HttpError;

    /// Catches the error using an [`ExceptionFilter`].
    ///
    /// If the filter returns `Ok(response)`, the error is replaced with
    /// the response body as the new error message.
    /// If the filter returns `Err(error)`, the original error passes through.
    ///
    /// # Errors
    ///
    /// Returns either the transformed error from the filter, or the original
    /// error if the filter does not catch it.
    ///
    /// ```ignore
    /// self.auth.login(&dto.username, &dto.password)
    ///     .catch(Arc::new(NotFoundFilter))?;
    /// ```
    fn exception_catch(self, filter: Arc<dyn ExceptionFilter>) -> Result<T, HttpError>;
}

impl<T> ExceptionExt<T> for Result<T, HttpError> {
    fn exception<F>(self, f: F) -> Result<T, HttpError>
    where
        F: FnOnce(HttpError) -> HttpError,
    {
        self.map_err(f)
    }

    fn exception_catch(self, filter: Arc<dyn ExceptionFilter>) -> Result<T, HttpError> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                let ctx = FilterContext::new(RouteMetadata::default());
                match filter.catch(&error, &ctx) {
                    Ok(response) => {
                        let body = String::from_utf8_lossy(response.body().as_bytes());
                        Err(HttpError::internal(error.code(), body.to_string()))
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
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
    ) -> Option<Result<Response, HttpError>> {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    struct TestFilter;
    impl ExceptionFilter for TestFilter {
        fn catch(
            &self,
            error: &HttpError,
            _context: &FilterContext,
        ) -> Result<Response, HttpError> {
            if error.code() == "HANDLED" {
                Ok(Response::empty(crate::HttpStatus::IM_A_TEAPOT))
            } else {
                Err(HttpError::internal("UNHANDLED", "not handled"))
            }
        }
    }

    struct FallbackFilter;
    impl ExceptionFilter for FallbackFilter {
        fn catch(
            &self,
            _error: &HttpError,
            _context: &FilterContext,
        ) -> Result<Response, HttpError> {
            Ok(Response::empty(crate::HttpStatus::OK))
        }
    }

    #[test]
    fn filter_context_new_stores_metadata() {
        let md = RouteMetadata::new();
        let ctx = FilterContext::new(md.clone());
        assert_eq!(ctx.route_metadata().is_empty(), md.is_empty());
    }

    #[test]
    fn exception_ext_ok_passes_through() {
        let result: Result<i32, HttpError> = Ok(42);
        let mapped = result.exception(|e| HttpError::internal("WRAPPED", e.message()));
        assert_eq!(mapped.unwrap(), 42);
    }

    #[test]
    fn exception_ext_err_transforms() {
        let result: Result<i32, HttpError> = Err(HttpError::bad_request("ORIG", "original"));
        let mapped = result.exception(|e| HttpError::internal("WRAPPED", e.message()));
        let err = mapped.unwrap_err();
        assert_eq!(err.code(), "WRAPPED");
        assert_eq!(err.message(), "original");
    }

    #[test]
    fn exception_catch_ok_passes_through() {
        let result: Result<i32, HttpError> = Ok(99);
        let filter = Arc::new(TestFilter);
        let mapped = result.exception_catch(filter);
        assert_eq!(mapped.unwrap(), 99);
    }

    #[test]
    fn exception_catch_handled_error_returns_internal_with_filter_body() {
        let result: Result<i32, HttpError> =
            Err(HttpError::bad_request("HANDLED", "handled error"));
        let filter = Arc::new(TestFilter);
        let err = result.exception_catch(filter).unwrap_err();
        assert_eq!(err.code(), "HANDLED");
    }

    #[test]
    fn exception_catch_unhandled_error_passes_through() {
        let result: Result<i32, HttpError> = Err(HttpError::bad_request("UNHANDLED", "unhandled"));
        let filter = Arc::new(TestFilter);
        let err = result.exception_catch(filter).unwrap_err();
        assert_eq!(err.code(), "UNHANDLED");
    }

    #[test]
    fn exception_filter_set_empty_returns_none() {
        let set = ExceptionFilterSet::new();
        let error = HttpError::bad_request("ERR", "err");
        let ctx = FilterContext::new(RouteMetadata::new());
        let result = set.catch(&error, &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn exception_filter_set_empty_len() {
        let set = ExceptionFilterSet::new();
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn exception_filter_set_push_increases_len() {
        let mut set = ExceptionFilterSet::new();
        set.push(Arc::new(TestFilter));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn exception_filter_set_catch_first_matching_filter() {
        let mut set = ExceptionFilterSet::new();
        set.push(Arc::new(TestFilter));
        set.push(Arc::new(FallbackFilter));
        let error = HttpError::bad_request("HANDLED", "handled");
        let ctx = FilterContext::new(RouteMetadata::new());
        let result = set.catch(&error, &ctx);
        assert!(result.is_some());
        let response = result.unwrap().unwrap();
        assert_eq!(response.status(), crate::HttpStatus::IM_A_TEAPOT);
    }

    #[test]
    fn exception_filter_set_catch_falls_through_to_second() {
        let mut set = ExceptionFilterSet::new();
        set.push(Arc::new(TestFilter));
        set.push(Arc::new(FallbackFilter));
        let error = HttpError::bad_request("OTHER", "other");
        let ctx = FilterContext::new(RouteMetadata::new());
        let result = set.catch(&error, &ctx);
        assert!(result.is_some());
        let response = result.unwrap();
        assert!(response.is_err());
        let err = response.unwrap_err();
        assert_eq!(err.status(), 500);
        assert_eq!(err.code(), "UNHANDLED");
    }

    #[test]
    fn exception_filter_set_append_merges_filters() {
        let mut set_a = ExceptionFilterSet::new();
        set_a.push(Arc::new(TestFilter));
        let mut set_b = ExceptionFilterSet::new();
        set_b.push(Arc::new(FallbackFilter));
        set_a.append(&mut set_b);
        assert_eq!(set_a.len(), 2);
    }

    #[test]
    fn exception_filter_set_is_cloneable() {
        let mut set = ExceptionFilterSet::new();
        set.push(Arc::new(TestFilter));
        let cloned = set.clone();
        assert_eq!(cloned.len(), 1);
    }
}
