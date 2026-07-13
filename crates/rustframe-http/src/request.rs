use std::collections::HashMap;

use http::Extensions;

use crate::{HeaderMap, HttpMethod, Uri};

/// An owned, transport-neutral HTTP request.
#[derive(Debug)]
pub struct FrameworkRequest {
    method: HttpMethod,
    uri: Uri,
    headers: HeaderMap,
    path_parameters: HashMap<String, String>,
    body: Vec<u8>,
}

impl FrameworkRequest {
    /// Creates a request from transport-owned parts.
    #[must_use]
    pub fn new(method: HttpMethod, uri: Uri, headers: HeaderMap, body: Vec<u8>) -> Self {
        Self {
            method,
            uri,
            headers,
            path_parameters: HashMap::new(),
            body,
        }
    }

    /// Attaches path parameters captured by the platform router.
    #[must_use]
    pub fn with_path_parameters(mut self, parameters: HashMap<String, String>) -> Self {
        self.path_parameters = parameters;
        self
    }

    /// Returns the request method.
    #[must_use]
    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    /// Returns the parsed URI.
    #[must_use]
    pub const fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Returns request headers.
    #[must_use]
    pub const fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns mutable access to request headers.
    #[must_use]
    pub const fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Returns one captured path parameter.
    #[must_use]
    pub fn path_parameter(&self, name: &str) -> Option<&str> {
        self.path_parameters.get(name).map(String::as_str)
    }

    /// Returns the raw request body.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

/// Mutable request state passed through extraction and handler dispatch.
#[derive(Debug)]
pub struct RequestContext {
    request: FrameworkRequest,
    extensions: Extensions,
}

impl RequestContext {
    /// Creates a context for a framework request.
    #[must_use]
    pub fn new(request: FrameworkRequest) -> Self {
        Self {
            request,
            extensions: Extensions::new(),
        }
    }

    /// Returns the transport-neutral request.
    #[must_use]
    pub const fn request(&self) -> &FrameworkRequest {
        &self.request
    }

    /// Returns mutable access to the transport-neutral request.
    #[must_use]
    pub const fn request_mut(&mut self) -> &mut FrameworkRequest {
        &mut self.request
    }

    /// Inserts typed request-scoped state and returns the previous value, if any.
    pub fn insert_extension<T: Clone + Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.extensions.insert(value)
    }

    /// Returns typed request-scoped state.
    #[must_use]
    pub fn extension<T: Clone + Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions.get::<T>()
    }
}
