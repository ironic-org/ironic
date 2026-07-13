use std::collections::HashMap;

use ironic_http::{
    CompiledHttpApplication, FrameworkRequest, FrameworkResponse, HeaderMap, HeaderName,
    HeaderValue, HttpError, HttpMethod, HttpStatus, IntoFrameworkResponse, RequestContext, Uri,
};
use serde::Serialize;

use crate::TestResponse;

/// A fluent in-process HTTP request builder.
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
        let request = FrameworkRequest::new(self.method, uri, self.headers, self.body)
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

fn error_response(error: HttpError) -> FrameworkResponse {
    error
        .into_framework_response()
        .unwrap_or_else(|_| FrameworkResponse::empty(HttpStatus::INTERNAL_SERVER_ERROR))
}
