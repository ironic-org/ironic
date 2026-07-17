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
            http.method = %context.request().method(),
            http.url = %context.request().uri(),
            http.status_code = tracing::field::Empty,
        );

        let span_clone = span.clone();
        Box::pin(
            async move {
                let mut response = next.run(context).await?;
                span_clone.record("http.status_code", response.status().as_u16());
                if let Ok(value) = HeaderValue::from_str(request_id.as_str()) {
                    response.headers_mut().insert(REQUEST_ID_HEADER, value);
                }
                Ok(response)
            }
            .instrument(span),
        )
    }
}

/// Logs HTTP request/response pairs as structured tracing events.
///
/// Captures method, URI, status code, body sizes, and duration. When the
/// `logging` feature is enabled on the `ironic` crate, these events are
/// automatically persisted by [`TimeSeriesLayer`].
///
/// Register on a [`CompiledHttpApplication`](crate::CompiledHttpApplication):
///
/// ```ignore
/// .middleware(RequestLogging::new())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestLogging;

impl RequestLogging {
    /// Creates the request logging middleware.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Middleware for RequestLogging {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let start = std::time::Instant::now();
            let method = context.request().method().clone();
            let uri = context.request().uri().clone();
            let req_body_size = context.request().body().len() as u64;

            let result = next.run(context).await;
            let duration = start.elapsed();
            let duration_ms = (duration.as_secs_f64() * 1000.0 * 100.0).round() / 100.0;

            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let resp_body_size = response.body().as_bytes().len() as u64;
                    let event_level = match status {
                        500..=599 => "error",
                        400..=499 => "warn",
                        _ => "info",
                    };
                    tracing::info!(
                        target: "ironic.http.access",
                        event_level,
                        http_method = %method,
                        http_uri = %uri,
                        http_status_code = status,
                        http_request_body_size = req_body_size,
                        http_response_body_size = resp_body_size,
                        http_duration_ms = duration_ms,
                    );
                }
                Err(error) => {
                    tracing::info!(
                        target: "ironic.http.access",
                        event_level = "error",
                        http_method = %method,
                        http_uri = %uri,
                        http_status_code = 500i64,
                        http_request_body_size = req_body_size,
                        http_response_body_size = 0u64,
                        http_duration_ms = duration_ms,
                        http_error_code = error.code(),
                    );
                }
            }

            result
        })
    }
}
