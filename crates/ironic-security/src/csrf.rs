//! CSRF protection middleware for Ironic.
//!
//! Feature flag: `security-csrf`.

use std::sync::Arc;

use ironic_http::{
    FrameworkResponse, HttpError, HttpMethod, HttpStatus, Middleware, MiddlewareNext, PipelineFuture,
    RequestContext,
};

/// CSRF protection configuration.
#[derive(Clone)]
pub struct CsrfConfig {
    cookie_name: String,
    header_name: String,
    token_generator: Arc<dyn Fn() -> String + Send + Sync>,
    safe_methods: Vec<HttpMethod>,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            cookie_name: "csrf-token".to_owned(),
            header_name: "x-csrf-token".to_owned(),
            token_generator: Arc::new(|| uuid::Uuid::new_v4().to_string()),
            safe_methods: vec![HttpMethod::GET, HttpMethod::HEAD, HttpMethod::OPTIONS],
        }
    }
}

impl CsrfConfig {
    /// Creates a new CSRF configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the cookie name for the CSRF token.
    #[must_use]
    pub fn cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Sets the header name for the CSRF token.
    #[must_use]
    pub fn header_name(mut self, name: impl Into<String>) -> Self {
        self.header_name = name.into();
        self
    }
}

/// CSRF protection middleware using the synchronizer token pattern.
///
/// The middleware expects a CSRF token in a cookie. For state-changing
/// requests (POST, PUT, PATCH, DELETE), it also expects the same token
/// in a request header. If the tokens don't match, a 403 response is
/// returned.
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

    fn extract_cookie_token(&self, context: &RequestContext) -> Option<String> {
        for cookie in context.request().headers().get_all(http::header::COOKIE) {
            if let Ok(value) = cookie.to_str() {
                for pair in value.split("; ") {
                    if let Some(token) = pair.strip_prefix(&format!("{}=", self.config.cookie_name))
                    {
                        return Some(token.to_owned());
                    }
                }
            }
        }
        None
    }

    fn extract_header_token(&self, context: &RequestContext) -> Option<String> {
        context
            .request()
            .headers()
            .get(&self.config.header_name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
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

            // Safe methods set the CSRF cookie if not present, but don't require validation
            if self.config.safe_methods.contains(method) {
                let cookie_token = self.extract_cookie_token(context);
                if cookie_token.is_none() {
                    let token = (self.config.token_generator)();
                    let mut response = next.run(context).await?;
                    response.headers_mut().insert(
                        http::header::SET_COOKIE,
                        http::HeaderValue::from_str(&format!(
                            "{}={}; Path=/; HttpOnly; SameSite=Strict",
                            self.config.cookie_name, token
                        ))
                        .unwrap_or_else(|_| {
                            http::HeaderValue::from_static("csrf-token=error; Path=/")
                        }),
                    );
                    return Ok(response);
                }
                return next.run(context).await;
            }

            let cookie_token = self.extract_cookie_token(context);
            let header_token = self.extract_header_token(context);

            match (cookie_token, header_token) {
                (Some(ref cookie), Some(ref header)) if cookie == header => {
                    next.run(context).await
                }
                _ => Err(HttpError::new(
                    HttpStatus::FORBIDDEN,
                    "RF_HTTP_CSRF_TOKEN_MISMATCH",
                    "CSRF token validation failed",
                )),
            }
        })
    }
}
