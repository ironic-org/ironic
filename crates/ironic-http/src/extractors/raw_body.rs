use crate::{ExtractFuture, ExtractedValue, ParameterExtractor, RequestContext};

/// Extracts the raw request body as `Vec<u8>`.
///
/// Use with the `#[raw_body]` parameter decorator:
///
/// ```ignore
/// #[post("/upload")]
/// async fn upload(#[raw_body] body: Vec<u8>) -> String {
///     format!("received {} bytes", body.len())
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RawBody;

impl ParameterExtractor for RawBody {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let body = context.request().body();
            let bytes: Vec<u8> = body.to_vec();
            Ok(Box::new(bytes) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "raw_body"
    }
}
