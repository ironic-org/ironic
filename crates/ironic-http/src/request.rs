use std::collections::HashMap;

use http::Extensions;

use crate::{HeaderMap, HttpMethod, RouteMetadata, Uri};

/// An owned, transport-neutral HTTP request.
#[derive(Debug)]
pub struct Request {
    method: HttpMethod,
    uri: Uri,
    headers: HeaderMap,
    path_parameters: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
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
pub struct RequestContext {
    request: Request,
    extensions: Extensions,
    route_metadata: Option<RouteMetadata>,
}

impl std::fmt::Debug for RequestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestContext")
            .field("request", &self.request)
            .field("route_metadata", &self.route_metadata.is_some())
            .finish_non_exhaustive()
    }
}

impl RequestContext {
    /// Creates a context for a framework request.
    #[must_use]
    pub fn new(request: Request) -> Self {
        Self {
            request,
            extensions: Extensions::new(),
            route_metadata: None,
        }
    }

    /// Attaches route metadata for interceptor and handler access.
    pub fn set_route_metadata(&mut self, metadata: RouteMetadata) {
        self.route_metadata = Some(metadata);
    }

    /// Returns the route metadata attached to the current request, if any.
    #[must_use]
    pub fn route_metadata(&self) -> Option<&RouteMetadata> {
        self.route_metadata.as_ref()
    }

    /// Returns the transport-neutral request.
    #[must_use]
    pub const fn request(&self) -> &Request {
        &self.request
    }

    /// Returns mutable access to the transport-neutral request.
    #[must_use]
    pub const fn request_mut(&mut self) -> &mut Request {
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

    /// Returns the client's preferred content type from the `Accept` header.
    ///
    /// Parses the header and returns the highest-weighted MIME type, or `None`
    /// if the header is absent.
    #[must_use]
    pub fn preferred_content_type(&self) -> Option<&str> {
        let accepting = self.request.headers().get("accept")?.to_str().ok()?;
        // Simple parser: take the first type before comma or semicolon
        let best = accepting
            .split(',')
            .next()?
            .split(';')
            .next()?
            .trim();
        if best.is_empty() { None } else { Some(best) }
    }

    /// Returns `true` if the client prefers JSON responses.
    #[must_use]
    pub fn accepts_json(&self) -> bool {
        self.preferred_content_type()
            .is_some_and(|ct| ct.contains("json") || ct.contains("*/*"))
    }
}
