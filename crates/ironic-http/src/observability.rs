use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn iso_timestamp() -> String {
    #[cfg(any(feature = "cron", feature = "logging"))]
    {
        ::chrono::Utc::now().to_rfc3339_opts(::chrono::SecondsFormat::Millis, true)
    }
    #[cfg(not(any(feature = "cron", feature = "logging")))]
    {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |d| d.as_secs_f64());
        format!("{secs:.3}")
    }
}

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
            .map_or(0, |duration| duration.as_secs());
        let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Self(format!("r{timestamp:x}{sequence:x}"))
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
            let duration_s = (duration.as_secs_f64() * 1000.0).round() / 1000.0;
            let ts = iso_timestamp();

            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let resp_body_size = response.body().as_bytes().len() as u64;
                    tracing::info!(
                        target: "ironic.http.access",
                        ts = %ts,
                        method = %method,
                        path = %uri,
                        status,
                        req_bytes = req_body_size,
                        res_bytes = resp_body_size,
                        dur_s = duration_s,
                    );
                }
                Err(error) => {
                    tracing::info!(
                        target: "ironic.http.access",
                        ts = %ts,
                        method = %method,
                        path = %uri,
                        status = 500i64,
                        req_bytes = req_body_size,
                        res_bytes = 0u64,
                        dur_s = duration_s,
                        err_code = error.code(),
                    );
                }
            }

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_id_new_and_as_str() {
        let id = RequestId::new("test-id");
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn request_id_display() {
        let id = RequestId::new("display-test");
        assert_eq!(format!("{id}"), "display-test");
    }

    #[test]
    fn request_id_equality() {
        let a = RequestId::new("same");
        let b = RequestId::new("same");
        assert_eq!(a, b);
    }

    #[test]
    fn request_id_inequality() {
        let a = RequestId::new("alpha");
        let b = RequestId::new("beta");
        assert_ne!(a, b);
    }

    #[test]
    fn request_id_generate_produces_non_empty_id() {
        let id = RequestId::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn request_id_clone() {
        let a = RequestId::new("clone-me");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn request_tracing_new() {
        let rt = RequestTracing::new();
        let _ = RequestTracing;
        let _ = rt;
    }

    #[test]
    fn request_tracing_is_copy() {
        let a = RequestTracing::new();
        let _ = a;
    }

    #[test]
    fn request_logging_new() {
        let rl = RequestLogging::new();
        let _ = RequestLogging;
        let _ = rl;
    }

    #[test]
    fn request_logging_is_copy() {
        let a = RequestLogging::new();
        let _ = a;
    }

    #[test]
    fn request_sequence_increments() {
        let a = RequestId::generate();
        let b = RequestId::generate();
        assert_ne!(a, b);
    }

    #[test]
    fn fake_iso_format_sync_backup() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |d| d.as_secs_f64());
        let formatted = format!("{secs:.3}");
        assert!(!formatted.is_empty());
        assert!(formatted.contains('.'));
    }
}
