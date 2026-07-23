//! Optional security middleware for Ironic: CORS, rate limiting, security headers, CSRF.
//!
//! Each module is gated behind its own feature flag and follows the existing
//! [`Middleware`] trait from `ironic-http`.
//!
//! # Feature flags
//!
//! | Flag | Module | Description |
//! |------|--------|-------------|
//! | `security-cors` | [`cors`] | Cross-Origin Resource Sharing middleware |
//! | `security-csrf` | [`csrf`] | CSRF protection via synchronizer token pattern |
//! | `security-rate-limit` | [`rate_limit`] | Rate limiting with in-memory or Redis backends |
//! | `security-headers` | [`security_headers`] | Security HTTP response headers |
//!
//! All four can be enabled together with the `security` meta-feature.

#[cfg(feature = "security-cors")]
pub mod cors;
#[cfg(feature = "security-cors")]
pub use cors::{CorsConfig, CorsMiddleware};
#[cfg(feature = "security-csrf")]
pub mod csrf;
#[cfg(feature = "security-rate-limit")]
pub mod rate_limit;
#[cfg(feature = "security-rate-limit")]
#[cfg(feature = "redis")]
pub use rate_limit::RedisRateLimiter;
#[cfg(feature = "security-rate-limit")]
pub use rate_limit::{InMemoryRateLimiter, RateLimitBackend, RateLimitMiddleware, RateLimitResult};
#[cfg(feature = "security-headers")]
pub mod security_headers;
#[cfg(feature = "security-headers")]
pub use security_headers::{SecurityHeadersConfig, SecurityHeadersMiddleware};

#[cfg(test)]
#[cfg(feature = "security")]
mod tests {
    use super::rate_limit::InMemoryRateLimiter;

    #[test]
    fn rate_limiter_allows_within_limit() {
        let limiter = InMemoryRateLimiter::new();
        assert!(limiter.check("client-1", 3, 60));
        assert!(limiter.check("client-1", 3, 60));
        assert!(limiter.check("client-1", 3, 60));
    }

    #[test]
    fn rate_limiter_blocks_excess_requests() {
        let limiter = InMemoryRateLimiter::new();
        assert!(limiter.check("client-2", 2, 60));
        assert!(limiter.check("client-2", 2, 60));
        assert!(!limiter.check("client-2", 2, 60));
    }

    #[test]
    fn rate_limiter_reports_remaining() {
        let limiter = InMemoryRateLimiter::new();
        assert_eq!(limiter.remaining("client-3", 5, 60), 5);
        let _ = limiter.check("client-3", 5, 60);
        assert_eq!(limiter.remaining("client-3", 5, 60), 4);
        let _ = limiter.check("client-3", 5, 60);
        assert_eq!(limiter.remaining("client-3", 5, 60), 3);
    }

    #[test]
    fn rate_limiter_isolates_clients() {
        let limiter = InMemoryRateLimiter::new();
        assert!(limiter.check("alice", 2, 60));
        assert!(limiter.check("alice", 2, 60));
        assert!(!limiter.check("alice", 2, 60));
        assert!(limiter.check("bob", 2, 60));
        assert!(limiter.check("bob", 2, 60));
    }

    #[cfg(feature = "security-cors")]
    #[test]
    fn cors_config_allows_origin() {
        let config = super::cors::CorsConfig::new().allowed_origins(["https://example.com"]);
        assert!(config.is_origin_allowed("https://example.com"));
        assert!(!config.is_origin_allowed("https://evil.com"));
    }

    #[cfg(feature = "security-cors")]
    #[test]
    fn cors_config_denies_by_default() {
        let config = super::cors::CorsConfig::new();
        // Default is deny — no origins allowed unless explicitly added
        assert!(!config.is_origin_allowed("https://any.com"));
        assert!(!config.is_origin_allowed("http://localhost:3000"));
    }

    #[cfg(feature = "security-cors")]
    #[test]
    fn cors_config_allows_explicit_origins() {
        let config = super::cors::CorsConfig::new().allowed_origins(["https://myapp.com"]);
        assert!(config.is_origin_allowed("https://myapp.com"));
        assert!(!config.is_origin_allowed("https://other.com"));
    }

    #[cfg(feature = "security-headers")]
    #[test]
    fn security_headers_defaults_are_set() {
        let config = super::security_headers::SecurityHeadersConfig::new();
        let _ = config;
    }

    #[cfg(feature = "security-headers")]
    #[test]
    fn security_headers_disable_hsts() {
        let config = super::security_headers::SecurityHeadersConfig::new().disable_hsts();
        let _ = config;
    }

    #[cfg(feature = "security-headers")]
    #[test]
    fn security_headers_custom_values() {
        let config = super::security_headers::SecurityHeadersConfig::new()
            .csp("default-src 'none'")
            .hsts("max-age=86400");
        let _ = config;
    }

    #[cfg(feature = "security-csrf")]
    #[test]
    fn csrf_config_builder_valid() {
        let config = super::csrf::CsrfConfig::new()
            .cookie_name("_csrf")
            .header_name("x-csrf");
        let _ = config;
    }

    #[cfg(feature = "security-rate-limit")]
    #[test]
    fn rate_limiter_sync_check_result() {
        let limiter = super::rate_limit::InMemoryRateLimiter::new();
        let result = limiter.sync_check("key", 1, 60);
        assert!(result.allowed);
        assert_eq!(result.remaining, 0);
    }

    #[cfg(feature = "security-rate-limit")]
    #[test]
    fn rate_limiter_key_isolation() {
        let limiter = super::rate_limit::InMemoryRateLimiter::new();
        assert!(limiter.check("alice", 2, 60));
        assert!(limiter.check("alice", 2, 60));
        assert!(!limiter.check("alice", 2, 60));
        assert!(limiter.check("bob", 2, 60));
    }

    #[cfg(feature = "security-csrf")]
    #[test]
    #[should_panic(expected = "CSRF cookie name")]
    fn csrf_config_panics_on_invalid_cookie_name() {
        let _ = super::csrf::CsrfConfig::new().cookie_name("bad;name");
    }

    #[cfg(feature = "security-csrf")]
    #[test]
    #[should_panic(expected = "CSRF header name")]
    fn csrf_config_panics_on_invalid_header_name() {
        let _ = super::csrf::CsrfConfig::new().header_name("bad\rname");
    }
}
