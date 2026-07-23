//! Security headers middleware for Ironic.
//!
//! Feature flag: `security-headers`.

use std::sync::Arc;

use ironic_http::{Middleware, MiddlewareNext, PipelineFuture, RequestContext};

/// Security headers configuration.
///
/// All headers are enabled by default with secure values.
/// Use the builder methods to customize or disable individual headers.
///
/// # Example
///
/// ```rust
/// use ironic::security::security_headers::SecurityHeadersConfig;
///
/// let config = SecurityHeadersConfig::new()
///     .hsts("max-age=63072000; includeSubDomains; preload")
///     .csp("default-src 'self'; script-src 'self' 'unsafe-inline'")
///     .disable_hsts();
/// ```
#[derive(Clone)]
pub struct SecurityHeadersConfig {
    hsts: Option<String>,
    csp: Option<String>,
    x_content_type_options: Option<String>,
    x_frame_options: Option<String>,
    referrer_policy: Option<String>,
    permissions_policy: Option<String>,
    cross_origin_opener_policy: Option<String>,
    cross_origin_embedder_policy: Option<String>,
    cross_origin_resource_policy: Option<String>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            hsts: Some("max-age=31536000; includeSubDomains".to_owned()),
            csp: Some("default-src 'self'".to_owned()),
            x_content_type_options: Some("nosniff".to_owned()),
            x_frame_options: Some("DENY".to_owned()),
            referrer_policy: Some("strict-origin-when-cross-origin".to_owned()),
            permissions_policy: Some("geolocation=()".to_owned()),
            cross_origin_opener_policy: Some("same-origin".to_owned()),
            cross_origin_embedder_policy: Some("require-corp".to_owned()),
            cross_origin_resource_policy: Some("same-origin".to_owned()),
        }
    }
}

impl SecurityHeadersConfig {
    /// Creates a new security headers configuration with secure defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `Strict-Transport-Security` header value.
    #[must_use]
    pub fn hsts(mut self, value: impl Into<String>) -> Self {
        self.hsts = Some(value.into());
        self
    }

    /// Sets the `Content-Security-Policy` header value.
    #[must_use]
    pub fn csp(mut self, value: impl Into<String>) -> Self {
        self.csp = Some(value.into());
        self
    }

    /// Sets the `X-Content-Type-Options` header value.
    #[must_use]
    pub fn x_content_type_options(mut self, value: impl Into<String>) -> Self {
        self.x_content_type_options = Some(value.into());
        self
    }

    /// Sets the `X-Frame-Options` header value.
    #[must_use]
    pub fn x_frame_options(mut self, value: impl Into<String>) -> Self {
        self.x_frame_options = Some(value.into());
        self
    }

    /// Sets the `Referrer-Policy` header value.
    #[must_use]
    pub fn referrer_policy(mut self, value: impl Into<String>) -> Self {
        self.referrer_policy = Some(value.into());
        self
    }

    /// Disables a header by setting it to `None`.
    #[must_use]
    pub fn disable_hsts(mut self) -> Self {
        self.hsts = None;
        self
    }

    /// Disables CSP header.
    #[must_use]
    pub fn disable_csp(mut self) -> Self {
        self.csp = None;
        self
    }

    /// Sets the `Cross-Origin-Opener-Policy` header value.
    #[must_use]
    pub fn cross_origin_opener_policy(mut self, value: impl Into<String>) -> Self {
        self.cross_origin_opener_policy = Some(value.into());
        self
    }

    /// Sets the `Cross-Origin-Embedder-Policy` header value.
    #[must_use]
    pub fn cross_origin_embedder_policy(mut self, value: impl Into<String>) -> Self {
        self.cross_origin_embedder_policy = Some(value.into());
        self
    }

    /// Sets the `Cross-Origin-Resource-Policy` header value.
    #[must_use]
    pub fn cross_origin_resource_policy(mut self, value: impl Into<String>) -> Self {
        self.cross_origin_resource_policy = Some(value.into());
        self
    }
}

fn insert_header(headers: &mut http::HeaderMap, name: http::HeaderName, value: &str) {
    if let Ok(v) = http::HeaderValue::from_str(value) {
        headers.insert(name, v);
    }
}

/// Middleware that sets security-related HTTP response headers.
///
/// Applies the configured headers to every response. Headers include
/// HSTS, CSP, X-Content-Type-Options, X-Frame-Options, Referrer-Policy,
/// Permissions-Policy, and Cross-Origin-* policies.
///
/// # Example
///
/// ```rust
/// use ironic::security::{SecurityHeadersConfig, SecurityHeadersMiddleware};
///
/// let middleware = SecurityHeadersMiddleware::new(SecurityHeadersConfig::new());
/// ```
#[derive(Clone)]
pub struct SecurityHeadersMiddleware {
    config: Arc<SecurityHeadersConfig>,
}

impl SecurityHeadersMiddleware {
    /// Creates a new security headers middleware.
    #[must_use]
    pub fn new(config: SecurityHeadersConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl Middleware for SecurityHeadersMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let mut response = next.run(context).await?;
            let headers = response.headers_mut();

            if let Some(ref val) = self.config.hsts {
                insert_header(headers, http::header::STRICT_TRANSPORT_SECURITY, val);
            }
            if let Some(ref val) = self.config.csp {
                insert_header(headers, http::header::CONTENT_SECURITY_POLICY, val);
            }
            if let Some(ref val) = self.config.x_content_type_options {
                insert_header(headers, http::header::X_CONTENT_TYPE_OPTIONS, val);
            }
            if let Some(ref val) = self.config.x_frame_options {
                insert_header(headers, http::header::X_FRAME_OPTIONS, val);
            }
            if let Some(ref val) = self.config.referrer_policy {
                insert_header(headers, http::header::REFERRER_POLICY, val);
            }
            if let Some(ref val) = self.config.permissions_policy {
                insert_header(
                    headers,
                    http::header::HeaderName::from_static("permissions-policy"),
                    val,
                );
            }
            if let Some(ref val) = self.config.cross_origin_opener_policy {
                insert_header(
                    headers,
                    http::header::HeaderName::from_static("cross-origin-opener-policy"),
                    val,
                );
            }
            if let Some(ref val) = self.config.cross_origin_embedder_policy {
                insert_header(
                    headers,
                    http::header::HeaderName::from_static("cross-origin-embedder-policy"),
                    val,
                );
            }
            if let Some(ref val) = self.config.cross_origin_resource_policy {
                insert_header(
                    headers,
                    http::header::HeaderName::from_static("cross-origin-resource-policy"),
                    val,
                );
            }

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_all_headers() {
        let config = SecurityHeadersConfig::new();
        assert!(config.hsts.is_some());
        assert!(config.csp.is_some());
        assert!(config.x_content_type_options.is_some());
        assert!(config.x_frame_options.is_some());
        assert!(config.referrer_policy.is_some());
    }

    #[test]
    fn disable_hsts_removes_header() {
        let config = SecurityHeadersConfig::new().disable_hsts();
        assert!(config.hsts.is_none());
    }

    #[test]
    fn disable_csp_removes_header() {
        let config = SecurityHeadersConfig::new().disable_csp();
        assert!(config.csp.is_none());
    }

    #[test]
    fn custom_hsts_value() {
        let config = SecurityHeadersConfig::new().hsts("max-age=63072000");
        assert_eq!(config.hsts.as_deref(), Some("max-age=63072000"));
    }

    #[test]
    fn custom_csp_value() {
        let config = SecurityHeadersConfig::new().csp("default-src 'none'");
        assert_eq!(config.csp.as_deref(), Some("default-src 'none'"));
    }

    #[test]
    fn custom_x_content_type_options() {
        let config = SecurityHeadersConfig::new().x_content_type_options("nosniff");
        assert_eq!(config.x_content_type_options.as_deref(), Some("nosniff"));
    }

    #[test]
    fn custom_x_frame_options() {
        let config = SecurityHeadersConfig::new().x_frame_options("SAMEORIGIN");
        assert_eq!(config.x_frame_options.as_deref(), Some("SAMEORIGIN"));
    }

    #[test]
    fn custom_referrer_policy() {
        let config = SecurityHeadersConfig::new().referrer_policy("no-referrer");
        assert_eq!(config.referrer_policy.as_deref(), Some("no-referrer"));
    }

    #[test]
    fn cross_origin_policies() {
        let config = SecurityHeadersConfig::new()
            .cross_origin_opener_policy("unsafe-none")
            .cross_origin_embedder_policy("unsafe-none")
            .cross_origin_resource_policy("cross-origin");
        assert_eq!(
            config.cross_origin_opener_policy.as_deref(),
            Some("unsafe-none")
        );
        assert_eq!(
            config.cross_origin_embedder_policy.as_deref(),
            Some("unsafe-none")
        );
        assert_eq!(
            config.cross_origin_resource_policy.as_deref(),
            Some("cross-origin")
        );
    }

    #[test]
    fn middleware_constructs() {
        let mw = SecurityHeadersMiddleware::new(SecurityHeadersConfig::new());
        let _ = mw;
    }

    #[test]
    fn default_hsts_value() {
        let config = SecurityHeadersConfig::new();
        assert_eq!(
            config.hsts.as_deref(),
            Some("max-age=31536000; includeSubDomains")
        );
    }

    #[test]
    fn default_csp_value() {
        let config = SecurityHeadersConfig::new();
        assert_eq!(config.csp.as_deref(), Some("default-src 'self'"));
    }
}
