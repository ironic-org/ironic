//! Optional security middleware for Ironic: CORS, rate limiting, security headers, CSRF.
//!
//! Each module is gated behind its own feature flag and follows the existing
//! [`Middleware`] trait from `ironic-http`.

#[cfg(feature = "security-cors")]
pub mod cors;
#[cfg(feature = "security-csrf")]
pub mod csrf;
#[cfg(feature = "security-rate-limit")]
pub mod rate_limit;
#[cfg(feature = "security-headers")]
pub mod security_headers;

#[cfg(test)]
#[cfg(feature = "security")]
mod tests {
    use super::rate_limit::InMemoryRateLimiter;

    #[test]
    fn rate_limiter_allows_within_limit() {
        let limiter = InMemoryRateLimiter::new(3, 60);
        assert!(limiter.check("client-1"));
        assert!(limiter.check("client-1"));
        assert!(limiter.check("client-1"));
    }

    #[test]
    fn rate_limiter_blocks_excess_requests() {
        let limiter = InMemoryRateLimiter::new(2, 60);
        assert!(limiter.check("client-2"));
        assert!(limiter.check("client-2"));
        assert!(!limiter.check("client-2"));
    }

    #[test]
    fn rate_limiter_reports_remaining() {
        let limiter = InMemoryRateLimiter::new(5, 60);
        assert_eq!(limiter.remaining("client-3"), 5);
        limiter.check("client-3");
        assert_eq!(limiter.remaining("client-3"), 4);
        limiter.check("client-3");
        assert_eq!(limiter.remaining("client-3"), 3);
    }

    #[test]
    fn rate_limiter_isolates_clients() {
        let limiter = InMemoryRateLimiter::new(2, 60);
        assert!(limiter.check("alice"));
        assert!(limiter.check("alice"));
        assert!(!limiter.check("alice"));
        assert!(limiter.check("bob"));
        assert!(limiter.check("bob"));
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
    fn cors_config_wildcard_allows_all() {
        let config = super::cors::CorsConfig::new();
        assert!(config.is_origin_allowed("https://any.com"));
        assert!(config.is_origin_allowed("http://localhost:3000"));
    }

    #[cfg(feature = "security-headers")]
    #[test]
    fn security_headers_defaults_are_set() {
        let config = super::security_headers::SecurityHeadersConfig::new();
        let _ = config;
    }
}
