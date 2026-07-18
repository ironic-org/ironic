// ── Pagination Decorator ────────────────────────────────────────
// Custom parameter extractor: ?page=1&size=20 → PaginationParams.
// Usage: #[decorator(Pagination)] on handler arguments.

use ironic::{ExtractFuture, ExtractedValue, ParameterExtractor, RequestContext};

#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub page: u64,
    pub size: u64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self { page: 1, size: 20 }
    }
}

pub struct Pagination;

impl Pagination {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ParameterExtractor for Pagination {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let query = context.request().uri().query().unwrap_or_default();
            let page = get_param(query, "page").unwrap_or(1);
            let size = get_param(query, "size").unwrap_or(20).min(100);
            Ok(Box::new(PaginationParams { page, size }) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "pagination (page, size)"
    }
}

fn get_param(query: &str, key: &str) -> Option<u64> {
    let prefix = format!("{key}=");
    query.split('&').find_map(|pair| {
        if pair.starts_with(&prefix) { pair[prefix.len()..].parse().ok() } else { None }
    })
}
