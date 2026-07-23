use std::collections::HashMap;

use ironic_http::{
    CompiledHttpApplication, HeaderMap, HeaderName, HeaderValue, HttpError, HttpMethod, HttpStatus,
    IntoResponse, Request, RequestContext, Response, Uri,
};
use serde::Serialize;

use crate::TestResponse;

/// A fluent in-process HTTP request builder.
///
/// Constructed by [`TestApplication::get()`], [`TestApplication::post()`], etc.
/// Provides chainable methods to set headers, body, and query parameters before
/// dispatching via [`send()`](TestRequestBuilder::send).
pub struct TestRequestBuilder<'a> {
    application: &'a CompiledHttpApplication,
    method: HttpMethod,
    path: String,
    headers: HeaderMap,
    body: Vec<u8>,
    error: Option<HttpError>,
}

impl<'a> TestRequestBuilder<'a> {
    pub(crate) fn new(
        application: &'a CompiledHttpApplication,
        method: HttpMethod,
        path: String,
    ) -> Self {
        Self {
            application,
            method,
            path,
            headers: HeaderMap::new(),
            body: Vec::new(),
            error: None,
        }
    }

    /// Adds a request header.
    ///
    /// Sets an internal error when the name or value is malformed — the error
    /// is returned when [`send()`](TestRequestBuilder::send) is called.
    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        match (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            (Ok(name), Ok(value)) => {
                self.headers.insert(name, value);
            }
            _ => {
                self.error = Some(HttpError::bad_request(
                    "RF_TEST_INVALID_HEADER",
                    format!("Invalid test request header `{name}`"),
                ));
            }
        }
        self
    }

    /// Serializes a request body as JSON and sets its content type.
    ///
    /// Sets an internal error when serialization fails — the error is returned
    /// when [`send()`](TestRequestBuilder::send) is called.
    #[must_use]
    pub fn json<T: Serialize + ?Sized>(mut self, value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(body) => {
                self.body = body;
                self.headers.insert(
                    ironic_http::HeaderName::from_static("content-type"),
                    ironic_http::HeaderValue::from_static("application/json"),
                );
            }
            Err(error) => {
                self.error = Some(HttpError::bad_request(
                    "RF_TEST_JSON_SERIALIZATION_FAILED",
                    format!("Could not serialize test request JSON: {error}"),
                ));
            }
        }
        self
    }

    /// Appends URL-encoded query parameters.
    ///
    /// Automatically chooses `?` or `&` as separator based on whether the path
    /// already contains a query string. Sets an internal error when
    /// serialization fails.
    #[must_use]
    pub fn query<T: Serialize + ?Sized>(mut self, value: &T) -> Self {
        match serde_urlencoded::to_string(value) {
            Ok(query) => {
                let separator = if self.path.contains('?') { '&' } else { '?' };
                self.path.push(separator);
                self.path.push_str(&query);
            }
            Err(error) => {
                self.error = Some(HttpError::bad_request(
                    "RF_TEST_QUERY_SERIALIZATION_FAILED",
                    format!("Could not serialize test query parameters: {error}"),
                ));
            }
        }
        self
    }

    /// Sets an arbitrary request body.
    #[must_use]
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Dispatches the request through the complete framework pipeline.
    ///
    /// If a previous builder call set an internal error (e.g. malformed header
    /// or failed JSON serialization), the error response is returned immediately
    /// without dispatching.
    pub async fn send(self) -> TestResponse {
        if let Some(error) = self.error {
            return TestResponse::new(error_response(error));
        }
        let uri: Uri = match self.path.parse() {
            Ok(uri) => uri,
            Err(error) => {
                return TestResponse::new(error_response(HttpError::bad_request(
                    "RF_TEST_INVALID_URI",
                    format!("Invalid test request URI: {error}"),
                )));
            }
        };
        let request_path = uri.path();
        let route = self.application.routes().iter().find_map(|route| {
            (route.method() == self.method)
                .then(|| match_path(route.path(), request_path))
                .flatten()
                .map(|parameters| (route, parameters))
        });
        let Some((route, parameters)) = route else {
            return TestResponse::new(error_response(HttpError::not_found(
                "RF_HTTP_ROUTE_NOT_FOUND",
                format!("No route matches `{} {request_path}`", self.method),
            )));
        };
        let request = Request::new(self.method, uri, self.headers, self.body)
            .with_path_parameters(parameters);
        let mut context = RequestContext::new(request);
        let response = self
            .application
            .execute(route, &mut context)
            .await
            .unwrap_or_else(error_response);
        TestResponse::new(response)
    }
}

fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern = segments(pattern);
    let path = segments(path);
    if pattern.len() != path.len() {
        return None;
    }
    let mut parameters = HashMap::new();
    for (expected, actual) in pattern.into_iter().zip(path) {
        if let Some(name) = expected.strip_prefix(':') {
            parameters.insert(name.to_owned(), actual.to_owned());
        } else if expected != actual {
            return None;
        }
    }
    Some(parameters)
}

fn segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn error_response(error: HttpError) -> Response {
    error
        .into_framework_response()
        .unwrap_or_else(|_| Response::empty(HttpStatus::INTERNAL_SERVER_ERROR))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Serialize;

    use super::{match_path, segments};

    #[test]
    fn segments_empty_path() {
        assert!(segments("").is_empty());
    }

    #[test]
    fn segments_root_path() {
        assert!(segments("/").is_empty());
    }

    #[test]
    fn segments_single() {
        assert_eq!(segments("/users"), vec!["users"]);
    }

    #[test]
    fn segments_multiple() {
        assert_eq!(segments("/users/42/posts"), vec!["users", "42", "posts"]);
    }

    #[test]
    fn segments_trailing_slash_is_ignored() {
        assert_eq!(segments("/users/"), vec!["users"]);
    }

    #[test]
    fn match_path_exact() {
        let result = match_path("/users/42", "/users/42");
        assert_eq!(result, Some(HashMap::new()));
    }

    #[test]
    fn match_path_with_parameter() {
        let result = match_path("/users/:id", "/users/42");
        let mut expected = HashMap::new();
        expected.insert("id".to_string(), "42".to_string());
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn match_path_multiple_parameters() {
        let result = match_path("/:a/:b/:c", "/x/y/z");
        let mut expected = HashMap::new();
        expected.insert("a".to_string(), "x".to_string());
        expected.insert("b".to_string(), "y".to_string());
        expected.insert("c".to_string(), "z".to_string());
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn match_path_mismatch_length() {
        let result = match_path("/users/:id", "/users/42/posts");
        assert!(result.is_none());
    }

    #[test]
    fn match_path_literal_mismatch() {
        let result = match_path("/users/:id", "/items/42");
        assert!(result.is_none());
    }

    #[test]
    fn header_sets_internal_error_on_invalid_name() {
        let builder = super::TestRequestBuilder {
            application: &create_empty_application(),
            method: ironic_http::HttpMethod::GET,
            path: "/".to_string(),
            headers: ironic_http::HeaderMap::new(),
            body: Vec::new(),
            error: None,
        };
        // Header name with spaces is invalid
        let builder = builder.header("invalid header name", "value");
        assert!(builder.error.is_some());
    }

    #[test]
    fn body_sets_bytes() {
        let builder = super::TestRequestBuilder {
            application: &create_empty_application(),
            method: ironic_http::HttpMethod::POST,
            path: "/".to_string(),
            headers: ironic_http::HeaderMap::new(),
            body: Vec::new(),
            error: None,
        };
        let builder = builder.body(vec![1, 2, 3]);
        assert_eq!(builder.body, vec![1, 2, 3]);
    }

    #[test]
    fn body_from_str() {
        let builder = super::TestRequestBuilder {
            application: &create_empty_application(),
            method: ironic_http::HttpMethod::POST,
            path: "/".to_string(),
            headers: ironic_http::HeaderMap::new(),
            body: Vec::new(),
            error: None,
        };
        let builder = builder.body("hello");
        assert_eq!(builder.body, b"hello");
    }

    #[test]
    fn query_appends_question_mark() {
        #[derive(Serialize)]
        struct Q {
            key: String,
        }
        let builder = super::TestRequestBuilder {
            application: &create_empty_application(),
            method: ironic_http::HttpMethod::GET,
            path: "/search".to_string(),
            headers: ironic_http::HeaderMap::new(),
            body: Vec::new(),
            error: None,
        };
        let builder = builder.query(&Q { key: "val".into() });
        assert_eq!(builder.path, "/search?key=val");
    }

    #[test]
    fn query_appends_ampersand_when_query_exists() {
        #[derive(Serialize)]
        struct Q {
            key: String,
        }
        let builder = super::TestRequestBuilder {
            application: &create_empty_application(),
            method: ironic_http::HttpMethod::GET,
            path: "/search?existing=1".to_string(),
            headers: ironic_http::HeaderMap::new(),
            body: Vec::new(),
            error: None,
        };
        let builder = builder.query(&Q { key: "val".into() });
        assert_eq!(builder.path, "/search?existing=1&key=val");
    }

    fn create_empty_application() -> ironic_http::CompiledHttpApplication {
        use ironic_di::ContainerBuilder;
        ironic_http::CompiledHttpApplication::new(ContainerBuilder::new().build(), Vec::new())
    }
}
