//! Built-in request parameter extractors.
//!
//! Currently provides the [`Pagination`] extractor for query-string paging.

pub(super) mod pagination;

pub use pagination::Pagination;

#[cfg(test)]
mod tests {
    #[test]
    fn pagination_type_is_accessible() {
        let _ = crate::Pagination::new();
    }
}
