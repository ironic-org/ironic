//! Standard error codes and response patterns for the Ironic framework.
//!
//! # Error Codes
//!
//! All error codes follow `CATEGORY_SPECIFIC` naming in SCREAMING_SNAKE_CASE:
//!
//! | Category | Prefix | Example |
//! |----------|--------|---------|
//! | Authentication | `AUTH_` | `AUTH_INVALID_CREDENTIALS` |
//! | Validation | `VALIDATION_` | `VALIDATION_FAILED` |
//! | Not Found | `NOT_FOUND_` | `NOT_FOUND_USER` |
//! | Conflict | `CONFLICT_` | `CONFLICT_EMAIL_EXISTS` |
//! | Rate Limit | `RATE_LIMIT_` | `RATE_LIMIT_EXCEEDED` |
//! | Internal | `INTERNAL_` | `INTERNAL_DATABASE_ERROR` |
//!
//! # Response Format
//!
//! **Error response** (always):
//! ```json
//! { "error": "AUTH_INVALID_CREDENTIALS", "message": "Invalid email or password" }
//! ```
//!
//! **Success response**: Use `Json<T>` directly, or `ApiResponse<T>` for enriched responses.

use serde::Serialize;

/// Well-known error codes. Prefer using these constants instead of raw strings.
///
/// # Examples
///
/// ```rust
/// use ironic::error_codes::codes;
///
/// assert_eq!(codes::AUTH_INVALID_CREDENTIALS, "AUTH_INVALID_CREDENTIALS");
/// assert_eq!(codes::NOT_FOUND, "NOT_FOUND");
/// ```
pub mod codes {
    // ── Authentication ────────────────────────────────────────────────
    /// Invalid email or password during login.
    pub const AUTH_INVALID_CREDENTIALS: &str = "AUTH_INVALID_CREDENTIALS";
    /// JWT token is missing, invalid, malformed, or expired.
    pub const AUTH_INVALID_TOKEN: &str = "AUTH_INVALID_TOKEN";
    /// Account with this email already exists.
    pub const AUTH_EMAIL_EXISTS: &str = "AUTH_EMAIL_EXISTS";
    /// The authenticated user lacks required permissions.
    pub const AUTH_FORBIDDEN: &str = "AUTH_FORBIDDEN";
    /// Authentication is required but no credentials were provided.
    pub const AUTH_UNAUTHORIZED: &str = "AUTH_UNAUTHORIZED";

    // ── Validation ────────────────────────────────────────────────────
    /// Request body or parameters failed validation rules.
    pub const VALIDATION_FAILED: &str = "VALIDATION_FAILED";

    // ── Not Found ─────────────────────────────────────────────────────
    /// The requested resource does not exist.
    pub const NOT_FOUND: &str = "NOT_FOUND";
    /// User with the given identifier was not found.
    pub const NOT_FOUND_USER: &str = "NOT_FOUND_USER";

    // ── Conflict ──────────────────────────────────────────────────────
    /// A resource with the same key already exists.
    pub const CONFLICT_DUPLICATE: &str = "CONFLICT_DUPLICATE";

    // ── Rate Limiting ─────────────────────────────────────────────────
    /// Too many requests — client exceeded their rate limit.
    pub const RATE_LIMIT_EXCEEDED: &str = "RATE_LIMIT_EXCEEDED";

    // ── Internal ──────────────────────────────────────────────────────
    /// An unexpected internal error occurred.
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    /// A database operation failed.
    pub const INTERNAL_DATABASE: &str = "INTERNAL_DATABASE";
    /// A hashing or cryptographic operation failed.
    pub const INTERNAL_HASH_ERROR: &str = "INTERNAL_HASH_ERROR";
    /// A JWT encoding or decoding operation failed unexpectedly.
    pub const INTERNAL_JWT_ERROR: &str = "INTERNAL_JWT_ERROR";
}

/// Standard success response wrapper with optional metadata.
///
/// Use for list endpoints where you want to include total count or pagination info.
/// For simple single-resource responses, use bare `Json<T>`.
///
/// # Examples
///
/// ```ignore
/// let resp = example();
/// assert_eq!(resp.data, vec![1, 2, 3]);
/// assert!(resp.total.is_none());
/// ```
///
/// ```json
/// { "data": [...], "total": 42, "page": 1 }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    /// The response payload.
    pub data: T,
    /// Total number of items (for paginated list endpoints).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// Current page number (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates a response with just the data payload (no pagination).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let resp = example();
    /// assert_eq!(resp.data, 42);
    /// ```
    pub fn new(data: T) -> Self {
        Self {
            data,
            total: None,
            page: None,
        }
    }

    /// Creates a paginated response with data, total count, and page number.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let resp = example();
    /// assert_eq!(resp.total, Some(42));
    /// assert_eq!(resp.page, Some(1));
    /// ```
    pub fn paginated(data: T, total: u64, page: u64) -> Self {
        Self {
            data,
            total: Some(total),
            page: Some(page),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_constants_have_expected_values() {
        assert_eq!(codes::AUTH_INVALID_CREDENTIALS, "AUTH_INVALID_CREDENTIALS");
        assert_eq!(codes::AUTH_INVALID_TOKEN, "AUTH_INVALID_TOKEN");
        assert_eq!(codes::AUTH_EMAIL_EXISTS, "AUTH_EMAIL_EXISTS");
        assert_eq!(codes::AUTH_FORBIDDEN, "AUTH_FORBIDDEN");
        assert_eq!(codes::AUTH_UNAUTHORIZED, "AUTH_UNAUTHORIZED");
        assert_eq!(codes::VALIDATION_FAILED, "VALIDATION_FAILED");
        assert_eq!(codes::NOT_FOUND, "NOT_FOUND");
        assert_eq!(codes::NOT_FOUND_USER, "NOT_FOUND_USER");
        assert_eq!(codes::CONFLICT_DUPLICATE, "CONFLICT_DUPLICATE");
        assert_eq!(codes::RATE_LIMIT_EXCEEDED, "RATE_LIMIT_EXCEEDED");
        assert_eq!(codes::INTERNAL_ERROR, "INTERNAL_ERROR");
        assert_eq!(codes::INTERNAL_DATABASE, "INTERNAL_DATABASE");
        assert_eq!(codes::INTERNAL_HASH_ERROR, "INTERNAL_HASH_ERROR");
        assert_eq!(codes::INTERNAL_JWT_ERROR, "INTERNAL_JWT_ERROR");
    }

    #[test]
    fn api_response_new_has_no_pagination() {
        let resp: ApiResponse<i32> = ApiResponse::new(42);
        assert_eq!(resp.data, 42);
        assert!(resp.total.is_none());
        assert!(resp.page.is_none());
    }

    #[test]
    fn api_response_paginated_sets_metadata() {
        let resp = ApiResponse::paginated(vec!["x", "y"], 100, 3);
        assert_eq!(resp.data, vec!["x", "y"]);
        assert_eq!(resp.total, Some(100));
        assert_eq!(resp.page, Some(3));
    }

    #[test]
    fn api_response_paginated_zero_count() {
        let resp = ApiResponse::paginated(Vec::<i32>::new(), 0, 1);
        assert!(resp.data.is_empty());
        assert_eq!(resp.total, Some(0));
        assert_eq!(resp.page, Some(1));
    }

    #[test]
    fn api_response_serializes_correctly() {
        let resp = ApiResponse::paginated(vec![1, 2], 10, 1);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains(r#""data":"#));
        assert!(json.contains(r#""total":10"#));
        assert!(json.contains(r#""page":1"#));
    }

    #[test]
    fn api_response_omits_optional_fields_when_none() {
        let resp: ApiResponse<i32> = ApiResponse::new(42);
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("total"));
        assert!(!json.contains("page"));
    }
}
