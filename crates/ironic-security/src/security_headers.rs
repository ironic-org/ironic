//! Security headers middleware for Ironic.
//!
//! Feature flag: `security-headers`.

use std::sync::Arc;

use ironic_http::{
    Middleware, MiddlewareNext, PipelineFuture, RequestContext,
};

/// Security headers configuration.
#[derive(Clone)]
pub struct SecurityHeadersConfig {
    hsts: Option<String>,
    csp: Option<String>,
    x_content_type_options: Option<String>,
    x_frame_options: Option<String>,
    referrer_policy: Option<String>,
    permissions_policy: Option<String>,
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
}

fn insert_header(headers: &mut http::HeaderMap, name: http::HeaderName, value: &str) {
    if let Ok(v) = http::HeaderValue::from_str(value) {
        headers.insert(name, v);
    }
}

/// Middleware that sets security-related HTTP response headers.
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

            Ok(response)
        })
    }
}
