//! CSRF protection middleware for Ironic.
//!
//! Uses the synchronizer token pattern: a cryptographically random token
//! is generated per-session, rendered into forms or supplied via header,
//! and validated on state-changing requests.
//!
//! Feature flag: `security-csrf`.

use std::sync::Arc;

use ironic_http::{HttpError, HttpStatus, Middleware, MiddlewareNext, PipelineFuture, RequestContext};

/// Configuration for CSRF protection.
#[derive(Clone, Debug)]
pub struct CsrfConfig {
    /// The header name that carries the CSRF token.
    pub header_name: String,
    /// The cookie name where the CSRF token is stored.
    pub cookie_name: String,
    /// HTTP methods that require CSRF validation.
    pub protected_methods: Vec<http::Method>,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            header_name: "x-csrf-token".into(),
            cookie_name: "csrf-token".into(),
            protected_methods: vec![
                http::Method::POST,
                http::Method::PUT,
                http::Method::PATCH,
                http::Method::DELETE,
            ],
        }
    }
}

/// Validates CSRF tokens on state-changing requests.
///
/// Uses the synchronizer token pattern: the server sets a cookie with a
/// cryptographically random token, and the client echoes it back via a
/// custom header. Tokens match → request is valid.
#[derive(Clone)]
pub struct CsrfMiddleware {
    config: Arc<CsrfConfig>,
}

impl CsrfMiddleware {
    /// Creates a new CSRF middleware.
    #[must_use]
    pub fn new(config: CsrfConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl Middleware for CsrfMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let method = context.request().method();

            if self.config.protected_methods.contains(method) {
                let cookie_header = context
                    .request()
                    .headers()
                    .get(http::header::COOKIE)
                    .and_then(|v| v.to_str().ok());

                let header_token = context
                    .request()
                    .headers()
                    .get(&self.config.header_name)
                    .and_then(|v| v.to_str().ok());

                let cookie_token = cookie_header.and_then(|cookies| {
                    cookies.split(';').find_map(|pair| {
                        let (key, value) = pair.trim().split_once('=')?;
                        if key == self.config.cookie_name {
                            Some(value.to_owned())
                        } else {
                            None
                        }
                    })
                });

                match (cookie_token, header_token) {
                    (Some(cookie), Some(header)) if cookie == header => {}
                    _ => {
                    return Err(HttpError::new(
                        HttpStatus::FORBIDDEN,
                        "RF_CSRF_TOKEN_MISMATCH",
                        "CSRF token validation failed.",
                    ));
                    }
                }
            }

            next.run(context).await
        })
    }
}
