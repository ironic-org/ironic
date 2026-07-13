//! CORS middleware for Ironic.
//!
//! Feature flag: `security-cors`.

use std::sync::Arc;

use ironic_http::{
    FrameworkResponse, HttpError, HttpMethod, HttpStatus, Middleware, MiddlewareNext,
    PipelineFuture, RequestContext,
};

/// CORS configuration.
#[derive(Clone)]
pub struct CorsConfig {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<HttpMethod>,
    allowed_headers: Vec<String>,
    allow_credentials: bool,
    max_age: Option<u64>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_owned()],
            allowed_methods: vec![
                HttpMethod::GET,
                HttpMethod::POST,
                HttpMethod::PUT,
                HttpMethod::PATCH,
                HttpMethod::DELETE,
                HttpMethod::OPTIONS,
                HttpMethod::HEAD,
            ],
            allowed_headers: vec![],
            allow_credentials: false,
            max_age: None,
        }
    }
}

impl CorsConfig {
    /// Creates a new CORS configuration allowing all origins.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets allowed origins.
    #[must_use]
    pub fn allowed_origins(mut self, origins: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.allowed_origins = origins.into_iter().map(Into::into).collect();
        self
    }

    /// Sets allowed methods.
    #[must_use]
    pub fn allowed_methods(mut self, methods: impl IntoIterator<Item = HttpMethod>) -> Self {
        self.allowed_methods = methods.into_iter().collect();
        self
    }

    /// Sets allowed headers.
    #[must_use]
    pub fn allowed_headers(mut self, headers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.allowed_headers = headers.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the `Access-Control-Allow-Credentials` header.
    #[must_use]
    pub const fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Sets the `Access-Control-Max-Age` header (seconds).
    #[must_use]
    pub const fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    pub(crate) fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|o| o == "*" || o == origin)
    }
}

/// Configurable CORS middleware.
#[derive(Clone)]
pub struct CorsMiddleware {
    config: Arc<CorsConfig>,
}

impl CorsMiddleware {
    /// Creates a new CORS middleware with the given configuration.
    #[must_use]
    pub fn new(config: CorsConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl Middleware for CorsMiddleware {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> PipelineFuture<'a> {
        Box::pin(async move {
            let origin = context
                .request()
                .headers()
                .get("origin")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_owned());

            let origin = match origin {
                Some(o) => o,
                None => return next.run(context).await,
            };

            if !self.config.is_origin_allowed(&origin) {
                return next.run(context).await;
            }

            let is_preflight = context.request().method() == HttpMethod::OPTIONS
                && context
                    .request()
                    .headers()
                    .get("access-control-request-method")
                    .is_some();

            if is_preflight {
                let mut response = FrameworkResponse::empty(HttpStatus::NO_CONTENT);
                set_cors_headers(&mut response, &self.config, &origin);
                return Ok(response);
            }

            let mut response = next.run(context).await?;
            set_cors_headers(&mut response, &self.config, &origin);
            Ok(response)
        })
    }
}

fn set_cors_headers(response: &mut FrameworkResponse, config: &CorsConfig, origin: &str) {
    let headers = response.headers_mut();
    if config.allowed_origins.contains(&"*".to_owned()) && !config.allow_credentials {
        headers.insert(
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            http::HeaderValue::from_static("*"),
        );
    } else {
        headers.insert(
            http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
            http::HeaderValue::from_str(origin)
                .unwrap_or_else(|_| http::HeaderValue::from_static("*")),
        );
        if config.allow_credentials {
            headers.insert(
                http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                http::HeaderValue::from_static("true"),
            );
        }
    }

    if !config.allowed_methods.is_empty() {
        let methods = config
            .allowed_methods
            .iter()
            .map(|m| m.as_str().to_owned())
            .collect::<Vec<_>>()
            .join(", ");
        if let Ok(value) = http::HeaderValue::from_str(&methods) {
            headers.insert(http::header::ACCESS_CONTROL_ALLOW_METHODS, value);
        }
    }

    if !config.allowed_headers.is_empty() {
        let header_str = config.allowed_headers.join(", ");
        if let Ok(value) = http::HeaderValue::from_str(&header_str) {
            headers.insert(http::header::ACCESS_CONTROL_ALLOW_HEADERS, value);
        }
    }

    if let Some(Ok(value)) = config
        .max_age
        .map(|s| http::HeaderValue::from_str(&s.to_string()))
    {
        headers.insert(http::header::ACCESS_CONTROL_MAX_AGE, value);
    }
}
