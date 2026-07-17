use std::sync::Arc;

use crate::http_impl::{
    CacheMetadata, HttpError, HttpStatus, Interceptor, InterceptorNext, PipelineFuture,
    RequestContext, Response,
};
use crate::services::cache::Cache;

/// An interceptor that checks a [`CacheMetadata`] annotation on the route
/// and serves cached responses when possible.
pub struct CacheInterceptor {
    backend: Arc<dyn Cache>,
}

impl CacheInterceptor {
    /// Wraps a cache backend.
    #[must_use]
    pub fn new(backend: Arc<dyn Cache>) -> Self {
        Self { backend }
    }
}

impl Interceptor for CacheInterceptor {
    fn intercept<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: InterceptorNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let cache_meta: Option<CacheMetadata> = context
                .route_metadata()
                .and_then(|meta| meta.get::<CacheMetadata>())
                .cloned();

            let Some(meta) = cache_meta else {
                return next.run(context).await;
            };

            let key = format!(
                "route-cache:{}:{}:{}",
                meta.ttl_secs,
                context.request().method(),
                context.request().uri().path(),
            );

            if let Some(cached) = self.backend.get(&key).await.map_err(|error| {
                HttpError::internal(
                    "IRONIC_CACHE_LOOKUP_FAILED",
                    format!("cache lookup failed: {error}"),
                )
            })? {
                return Ok(Response::bytes(HttpStatus::OK, cached));
            }

            let response = next.run(context).await?;
            let body_bytes = response.body().as_bytes().to_vec();

            self.backend
                .set(
                    &key,
                    body_bytes,
                    Some(std::time::Duration::from_secs(meta.ttl_secs)),
                )
                .await
                .map_err(|error| {
                    HttpError::internal(
                        "IRONIC_CACHE_STORE_FAILED",
                        format!("cache store failed: {error}"),
                    )
                })?;

            Ok(response)
        })
    }
}
