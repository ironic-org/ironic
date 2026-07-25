use std::marker::PhantomData;

use crate::{ExtractFuture, ExtractedValue, ParameterExtractor, RequestContext, HttpError};

/// Extracts a named cookie value from the request.
///
/// Use with the `#[cookie]` parameter decorator:
///
/// ```ignore
/// #[get("/profile")]
/// async fn profile(#[cookie("session_id")] session_id: String) -> String {
///     format!("session: {session_id}")
/// }
/// ```
#[derive(Debug)]
pub struct CookieParameter<T> {
    name: &'static str,
    marker: PhantomData<fn() -> T>,
}

impl<T> CookieParameter<T> {
    /// Creates a new cookie extractor for the given cookie name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            marker: PhantomData,
        }
    }
}

impl ParameterExtractor for CookieParameter<String> {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let cookies = context
                .request()
                .headers()
                .get("cookie")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            for pair in cookies.split(';') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once('=') && key.trim() == self.name {
                    return Ok(Box::new(value.trim().to_string()) as ExtractedValue);
                }
            }

            Err(HttpError::bad_request(
                "RF_HTTP_MISSING_COOKIE",
                format!("cookie `{}` not found", self.name),
            ))
        })
    }

    fn description(&self) -> &'static str {
        self.name
    }
}
