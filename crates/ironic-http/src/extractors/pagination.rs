//! Built-in pagination extractor for list endpoints.
//!
//! Parses `?page=N&size=M` from the query string using proper URL decoding.
//! Defaults: page=1, size=20, `max_size=100`.
//!
//! Usage:
//! ```ignore
//! use ironic::Pagination;
//!
//! #[get("")]
//! async fn list(&self, #[decorator(Pagination)] p: Pagination) -> Response {
//!     let posts = self.service.list(p.offset(), p.limit()).await?;
//!     Response::json(200, &posts)
//! }
//! ```

use crate::{ExtractFuture, ExtractedValue, ParameterExtractor, RequestContext};

/// Parsed pagination parameters from `?page=N&size=M`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pagination {
    /// The current page number (1-based, minimum 1).
    pub page: u64,
    /// The number of items per page (minimum 1, maximum `max_size`).
    pub size: u64,
    max_size: u64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            size: 20,
            max_size: 100,
        }
    }
}

impl Pagination {
    /// Creates a `Pagination` instance with `Default::default()` settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a maximum page size (clamped on extraction).
    pub fn max_size(mut self, max: u64) -> Self {
        self.max_size = max;
        self
    }

    /// SQL offset: `(page - 1) * size`.
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.size
    }

    /// SQL limit.
    pub fn limit(&self) -> u64 {
        self.size
    }
}

impl ParameterExtractor for Pagination {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let query = context
                .request()
                .uri()
                .query()
                .unwrap_or_default();

            let mut page: u64 = 1;
            let mut size: u64 = 20;
            let max = self.max_size;

            for (key, value) in serde_urlencoded::from_str::<std::vec::Vec<(String, String)>>(query)
                .unwrap_or_default() {
                match key.as_str() {
                    "page" => {
                        if let Ok(p) = value.parse::<u64>() {
                            page = p.max(1);
                        }
                    }
                    "size" => {
                        if let Ok(s) = value.parse::<u64>() {
                            size = s.max(1).min(max);
                        }
                    }
                    _ => {}
                }
            }

            Ok(Box::new(Pagination {
                page,
                size,
                max_size: max,
            }) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "pagination"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Request;
    use http::{HeaderMap, Method, Uri};

    fn context(query: &str) -> RequestContext {
        let uri = format!("/items?{query}").parse::<Uri>().unwrap();
        RequestContext::new(Request::new(
            Method::GET,
            uri,
            HeaderMap::new(),
            Vec::new(),
        ))
    }

    #[tokio::test]
    async fn defaults_when_missing() {
        let pagination: Pagination = Pagination::new()
            .extract(&mut context(""))
            .await
            .unwrap()
            .downcast::<Pagination>()
            .map(|b| *b)
            .unwrap();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.size, 20);
    }

    #[tokio::test]
    async fn parses_page_and_size() {
        let pagination: Pagination = Pagination::new()
            .extract(&mut context("page=3&size=10"))
            .await
            .unwrap()
            .downcast::<Pagination>()
            .map(|b| *b)
            .unwrap();
        assert_eq!(pagination.page, 3);
        assert_eq!(pagination.size, 10);
        assert_eq!(pagination.offset(), 20);
        assert_eq!(pagination.limit(), 10);
    }

    #[tokio::test]
    async fn clamps_to_max_size() {
        let pagination: Pagination = Pagination::new()
            .max_size(50)
            .extract(&mut context("size=200"))
            .await
            .unwrap()
            .downcast::<Pagination>()
            .map(|b| *b)
            .unwrap();
        assert_eq!(pagination.size, 50);
    }

    #[tokio::test]
    async fn clamps_page_to_one() {
        let pagination: Pagination = Pagination::new()
            .extract(&mut context("page=0&size=10"))
            .await
            .unwrap()
            .downcast::<Pagination>()
            .map(|b| *b)
            .unwrap();
        assert_eq!(pagination.page, 1);
    }

    #[tokio::test]
    async fn offset_calculates_correctly() {
        let p = Pagination {
            page: 2,
            size: 20,
            max_size: 100,
        };
        assert_eq!(p.offset(), 20);
        assert_eq!(p.limit(), 20);
    }
}
