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
        let best = accepting.split(',').next()?.split(';').next()?.trim();
        if best.is_empty() { None } else { Some(best) }
    }

    /// Returns `true` if the client prefers JSON responses.
    #[must_use]
    pub fn accepts_json(&self) -> bool {
        self.preferred_content_type()
            .is_some_and(|ct| ct.contains("json") || ct.contains("*/*"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn get_request(uri: &str) -> Request {
        Request::new(
            HttpMethod::GET,
            uri.parse::<Uri>().unwrap(),
            HeaderMap::new(),
            Vec::new(),
        )
    }

    #[test]
    fn request_new_sets_method_uri_headers_body() {
        let req = Request::new(
            HttpMethod::POST,
            "/test".parse::<Uri>().unwrap(),
            HeaderMap::new(),
            vec![1, 2, 3],
        );
        assert_eq!(req.method(), &HttpMethod::POST);
        assert_eq!(req.uri(), &"/test".parse::<Uri>().unwrap());
        assert_eq!(req.body(), &[1, 2, 3]);
    }

    #[test]
    fn request_with_path_parameters() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "42".to_string());
        let req = get_request("/users/42").with_path_parameters(params);
        assert_eq!(req.path_parameter("id"), Some("42"));
    }

    #[test]
    fn request_path_parameter_missing() {
        let req = get_request("/users");
        assert_eq!(req.path_parameter("id"), None);
    }

    #[test]
    fn request_headers_mut_allows_modification() {
        let mut req = get_request("/test");
        req.headers_mut()
            .insert("x-custom", "value".parse().unwrap());
        assert_eq!(
            req.headers().get("x-custom").unwrap().to_str().unwrap(),
            "value"
        );
    }

    #[test]
    fn request_context_new_creates_empty_context() {
        let ctx = RequestContext::new(get_request("/test"));
        assert_eq!(ctx.request().method(), &HttpMethod::GET);
        assert!(ctx.route_metadata().is_none());
    }

    #[test]
    fn request_context_set_route_metadata() {
        let mut ctx = RequestContext::new(get_request("/test"));
        let md = RouteMetadata::new();
        ctx.set_route_metadata(md.clone());
        assert!(ctx.route_metadata().is_some());
    }

    #[test]
    fn request_context_extension_round_trip() {
        let mut ctx = RequestContext::new(get_request("/test"));
        ctx.insert_extension(42u32);
        assert_eq!(ctx.extension::<u32>(), Some(&42));
    }

    #[test]
    fn request_context_extension_overwrite() {
        let mut ctx = RequestContext::new(get_request("/test"));
        ctx.insert_extension(1u32);
        let prev = ctx.insert_extension(2u32);
        assert_eq!(prev, Some(1u32));
        assert_eq!(ctx.extension::<u32>(), Some(&2));
    }

    #[test]
    fn request_context_extension_none_when_missing() {
        let ctx = RequestContext::new(get_request("/test"));
        assert_eq!(ctx.extension::<u32>(), None);
    }

    #[test]
    fn request_context_preferred_content_type_none_when_missing() {
        let ctx = RequestContext::new(get_request("/test"));
        assert_eq!(ctx.preferred_content_type(), None);
    }

    #[test]
    fn request_context_preferred_content_type_returns_value() {
        let mut req = get_request("/test");
        req.headers_mut()
            .insert("accept", "application/json".parse().unwrap());
        let ctx = RequestContext::new(req);
        assert_eq!(
            ctx.preferred_content_type(),
            Some("application/json")
        );
    }

    #[test]
    fn request_context_accepts_json_with_json_content_type() {
        let mut req = get_request("/test");
        req.headers_mut()
            .insert("accept", "application/json".parse().unwrap());
        let ctx = RequestContext::new(req);
        assert!(ctx.accepts_json());
    }

    #[test]
    fn request_context_accepts_json_with_wildcard() {
        let mut req = get_request("/test");
        req.headers_mut()
            .insert("accept", "*/*".parse().unwrap());
        let ctx = RequestContext::new(req);
        assert!(ctx.accepts_json());
    }

    #[test]
    fn request_context_accepts_json_false_for_xml() {
        let mut req = get_request("/test");
        req.headers_mut()
            .insert("accept", "application/xml".parse().unwrap());
        let ctx = RequestContext::new(req);
        assert!(!ctx.accepts_json());
    }

    #[test]
    fn request_context_request_mut_allows_mutation() {
        let mut ctx = RequestContext::new(get_request("/test"));
        ctx.request_mut()
            .headers_mut()
            .insert("x-foo", "bar".parse().unwrap());
        assert_eq!(
            ctx.request()
                .headers()
                .get("x-foo")
                .unwrap()
                .to_str()
                .unwrap(),
            "bar"
        );
    }
}
