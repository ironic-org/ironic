//! Built-in request parameter extractors.

pub(super) mod cookie;
pub(super) mod pagination;
pub(super) mod raw_body;

pub use cookie::CookieParameter;
pub use pagination::Pagination;
pub use raw_body::RawBody;

#[cfg(test)]
mod tests {
    #[test]
    fn pagination_type_is_accessible() {
        let _ = crate::Pagination::new();
    }
}
