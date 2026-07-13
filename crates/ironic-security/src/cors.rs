//! CORS middleware for Ironic.
//!
//! Feature flag: `security-cors`.

use ironic_http::{Middleware, MiddlewareNext, PipelineFuture, RequestContext};

/// Configurable CORS middleware.
#[derive(Clone)]
pub struct CorsMiddleware;

impl CorsMiddleware {
    /// Creates a new CORS middleware.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for CorsMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move { next.run(context).await })
    }
}
