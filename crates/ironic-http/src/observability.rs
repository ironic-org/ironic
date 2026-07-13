use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use tracing::Instrument;

use crate::{HeaderName, HeaderValue, Middleware, MiddlewareNext, PipelineFuture, RequestContext};

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// A request correlation identifier available through [`RequestContext::extension`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RequestId(String);

impl RequestId {
    /// Creates an application-supplied request identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the identifier string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn generate() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Self(format!("rf-{timestamp:032x}-{sequence:016x}"))
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Adds request IDs, response correlation headers, and structured tracing spans.
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestTracing;

impl RequestTracing {
    /// Creates the tracing middleware.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Middleware for RequestTracing {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        let request_id = context
            .request()
            .headers()
            .get(&REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.is_empty())
            .map_or_else(RequestId::generate, RequestId::new);
        context.insert_extension(request_id.clone());
        let span = tracing::info_span!(
            "ironic.http.request",
            request_id = %request_id,
            method = %context.request().method(),
            uri = %context.request().uri(),
        );

        Box::pin(
            async move {
                let mut response = next.run(context).await?;
                if let Ok(value) = HeaderValue::from_str(request_id.as_str()) {
                    response.headers_mut().insert(REQUEST_ID_HEADER, value);
                }
                Ok(response)
            }
            .instrument(span),
        )
    }
}
