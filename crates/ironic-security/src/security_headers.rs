//! Security headers middleware for Ironic.
//!
//! Adds HSTS, CSP, X-Content-Type-Options, and X-Frame-Options headers
//! to every response.
//!
//! Feature flag: `security-headers`.

use std::sync::Arc;

use ironic_http::{HeaderName, HeaderValue, Middleware, MiddlewareNext, PipelineFuture, RequestContext};

/// Security header values applied to all responses.
#[derive(Clone, Debug)]
pub struct SecurityHeadersConfig {
    /// `Strict-Transport-Security` header value (e.g., "max-age=31536000; includeSubDomains").
    pub hsts: Option<String>,
    /// `Content-Security-Policy` header value.
    pub csp: Option<String>,
    /// `X-Content-Type-Options` header value (typically "nosniff").
    pub x_content_type_options: Option<String>,
    /// `X-Frame-Options` header value (e.g., "DENY" or "SAMEORIGIN").
    pub x_frame_options: Option<String>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            hsts: Some("max-age=31536000; includeSubDomains".into()),
            csp: None,
            x_content_type_options: Some("nosniff".into()),
            x_frame_options: Some("DENY".into()),
        }
    }
}

/// Middleware that applies security headers to all responses.
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

            if let Some(hsts) = &self.config.hsts {
                headers.insert(
                    HeaderName::from_static("strict-transport-security"),
                    HeaderValue::from_str(hsts).unwrap(),
                );
            }
            if let Some(csp) = &self.config.csp {
                headers.insert(
                    HeaderName::from_static("content-security-policy"),
                    HeaderValue::from_str(csp).unwrap(),
                );
            }
            if let Some(xcto) = &self.config.x_content_type_options {
                headers.insert(
                    HeaderName::from_static("x-content-type-options"),
                    HeaderValue::from_str(xcto).unwrap(),
                );
            }
            if let Some(xfo) = &self.config.x_frame_options {
                headers.insert(
                    HeaderName::from_static("x-frame-options"),
                    HeaderValue::from_str(xfo).unwrap(),
                );
            }

            Ok(response)
        })
    }
}
