//! Optional security middleware for Ironic: CORS, rate limiting, security headers, CSRF.
//!
//! Each module is gated behind its own feature flag and follows the existing
//! [`Middleware`] trait from `ironic-http`.

#[cfg(feature = "security-cors")]
pub mod cors;
#[cfg(feature = "security-rate-limit")]
pub mod rate_limit;
#[cfg(feature = "security-headers")]
pub mod security_headers;
#[cfg(feature = "security-csrf")]
pub mod csrf;
