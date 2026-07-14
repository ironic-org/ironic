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
    /// Creates a response with just the data payload.
    pub fn new(data: T) -> Self {
        Self {
            data,
            total: None,
            page: None,
        }
    }

    /// Creates a paginated response with data, total count, and page number.
    pub fn paginated(data: T, total: u64, page: u64) -> Self {
        Self {
            data,
            total: Some(total),
            page: Some(page),
        }
    }
}
