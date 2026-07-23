use std::sync::Arc;

use serde::Serialize;

use crate::{HeaderMap, HeaderValue, HttpError, HttpStatus};

/// An owned transport-neutral response body.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum Body {
    /// No response body.
    #[default]
    Empty,
    /// A complete in-memory response body.
    Bytes(Vec<u8>),
    /// A streaming response body using shared ownership for efficient cloning.
    Stream(Arc<Vec<u8>>),
}

impl Body {
    /// Returns the body as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Empty => &[],
            Self::Bytes(bytes) => bytes,
            Self::Stream(bytes) => bytes,
        }
    }
}

/// An owned transport-neutral HTTP response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    status: HttpStatus,
    headers: HeaderMap,
    body: Body,
}

impl Response {
    /// Creates an empty response with `status`.
    #[must_use]
    pub fn empty(status: HttpStatus) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Body::Empty,
        }
    }

    /// Creates a byte response.
    #[must_use]
    pub fn bytes(status: HttpStatus, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Body::Bytes(body.into()),
        }
    }

    /// Creates a streaming response using a shared body for efficient cloning.
    #[must_use]
    pub fn from_stream(status: HttpStatus, body: Arc<Vec<u8>>) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Body::Stream(body),
        }
    }

    /// Serializes a successful JSON response.
    ///
    /// # Errors
    ///
    /// Returns an internal [`HttpError`] when serialization fails.
    pub fn json<T: Serialize>(status: HttpStatus, value: &T) -> Result<Self, HttpError> {
        let body = serde_json::to_vec(value).map_err(|_| {
            HttpError::internal(
                "RF_HTTP_SERIALIZATION_FAILED",
                "Response serialization failed",
            )
        })?;
        let mut response = Self::bytes(status, body);
        response.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        Ok(response)
    }

    /// Creates a structured JSON error response.
    #[must_use]
    pub fn error(status: HttpStatus, code: &'static str, message: impl Into<String>) -> Self {
        #[derive(Serialize)]
        struct ErrorBody<'a> {
            status: u16,
            code: &'a str,
            message: String,
        }

        let body = ErrorBody {
            status: status.as_u16(),
            code,
            message: message.into(),
        };
        Self::json(status, &body).unwrap_or_else(|_| Self::empty(status))
    }

    /// Creates a structured JSON error response with request tracing metadata.
    ///
    /// Includes `timestamp` (Unix millis) and `request_id` in the response body
    /// for production error correlation with server logs.
    #[must_use]
    pub fn error_with_tracing(
        status: HttpStatus,
        code: &'static str,
        message: impl Into<String>,
        request_id: Option<&str>,
    ) -> Self {
        #[derive(Serialize)]
        struct ErrorBody<'a> {
            status: u16,
            code: &'a str,
            message: String,
            timestamp_ms: u128,
            #[serde(skip_serializing_if = "Option::is_none")]
            request_id: Option<&'a str>,
        }

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis());

        let body = ErrorBody {
            status: status.as_u16(),
            code,
            message: message.into(),
            timestamp_ms: ts,
            request_id,
        };
        Self::json(status, &body).unwrap_or_else(|_| Self::empty(status))
    }

    /// Paginated response wrapper.
    ///
    /// Serializes as `{"items": [...], "total": N, "offset": N, "limit": N}`.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when serialization fails.
    pub fn paginated<T: Serialize>(
        items: &[T],
        total: u64,
        offset: u64,
        limit: u64,
    ) -> Result<Self, HttpError> {
        #[derive(Serialize)]
        struct PageBody<'a, I> {
            items: &'a [I],
            total: u64,
            offset: u64,
            limit: u64,
        }

        Self::json(
            HttpStatus::OK,
            &PageBody {
                items,
                total,
                offset,
                limit,
            },
        )
    }

    /// Returns the response status.
    #[must_use]
    pub const fn status(&self) -> HttpStatus {
        self.status
    }

    /// Returns response headers.
    #[must_use]
    pub const fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns mutable response headers.
    #[must_use]
    pub const fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Returns the response body.
    #[must_use]
    pub const fn body(&self) -> &Body {
        &self.body
    }

    /// Splits the response into transport-owned parts.
    #[must_use]
    pub fn into_parts(self) -> (HttpStatus, HeaderMap, Body) {
        (self.status, self.headers, self.body)
    }

    /// Replaces the response body.
    pub fn set_body(&mut self, body: Body) {
        self.body = body;
    }
}

/// Converts a handler result into a framework response.
pub trait IntoResponse {
    /// Performs response conversion.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when conversion or serialization fails.
    fn into_framework_response(self) -> Result<Response, HttpError>;
}

impl IntoResponse for Response {
    fn into_framework_response(self) -> Result<Response, HttpError> {
        Ok(self)
    }
}

impl IntoResponse for () {
    fn into_framework_response(self) -> Result<Response, HttpError> {
        Ok(Response::empty(HttpStatus::NO_CONTENT))
    }
}

impl IntoResponse for String {
    fn into_framework_response(self) -> Result<Response, HttpError> {
        Ok(Response::bytes(HttpStatus::OK, self))
    }
}

impl IntoResponse for &'static str {
    fn into_framework_response(self) -> Result<Response, HttpError> {
        self.to_owned().into_framework_response()
    }
}

/// Marks a value for JSON response serialization.
#[derive(Clone, Copy, Debug)]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_framework_response(self) -> Result<Response, HttpError> {
        Response::json(HttpStatus::OK, &self.0)
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_framework_response(self) -> Result<Response, HttpError> {
        match self {
            Ok(value) => value.into_framework_response(),
            Err(error) => error.into_framework_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpStatus;
    use std::sync::Arc;
    use http;

    #[test]
    fn body_as_bytes_empty() {
        assert!(Body::Empty.as_bytes().is_empty());
    }

    #[test]
    fn body_as_bytes_bytes() {
        let body = Body::Bytes(vec![1, 2, 3]);
        assert_eq!(body.as_bytes(), &[1, 2, 3]);
    }

    #[test]
    fn body_as_bytes_stream() {
        let body = Body::Stream(Arc::new(vec![4, 5, 6]));
        assert_eq!(body.as_bytes(), &[4, 5, 6]);
    }

    #[test]
    fn body_default_is_empty() {
        let body: Body = Body::default();
        assert_eq!(body, Body::Empty);
    }

    #[test]
    fn response_empty_creates_correct_status() {
        let resp = Response::empty(HttpStatus::NO_CONTENT);
        assert_eq!(resp.status(), HttpStatus::NO_CONTENT);
        assert_eq!(resp.body(), &Body::Empty);
    }

    #[test]
    fn response_bytes_creates_correct_body() {
        let resp = Response::bytes(HttpStatus::OK, vec![10, 20]);
        assert_eq!(resp.status(), HttpStatus::OK);
        assert_eq!(resp.body().as_bytes(), &[10, 20]);
    }

    #[test]
    fn response_bytes_from_str() {
        let resp = Response::bytes(HttpStatus::OK, "hello");
        assert_eq!(resp.body().as_bytes(), b"hello");
    }

    #[test]
    fn response_from_stream_creates_streaming_body() {
        let data = Arc::new(vec![1, 2, 3]);
        let resp = Response::from_stream(HttpStatus::OK, Arc::clone(&data));
        assert_eq!(resp.body(), &Body::Stream(data));
    }

    #[test]
    fn response_json_serializes_and_sets_content_type() {
        let resp = Response::json(HttpStatus::OK, &serde_json::json!({"key": "value"})).unwrap();
        assert_eq!(resp.status(), HttpStatus::OK);
        assert_eq!(
            resp.headers()
                .get(http::header::CONTENT_TYPE)
                .unwrap()
                .to_str()
                .unwrap(),
            "application/json"
        );
        let body: serde_json::Value =
            serde_json::from_slice(resp.body().as_bytes()).unwrap();
        assert_eq!(body["key"], "value");
    }

    #[test]
    fn response_error_creates_structured_error() {
        let resp = Response::error(HttpStatus::NOT_FOUND, "NOT_FOUND", "resource missing");
        assert_eq!(resp.status(), HttpStatus::NOT_FOUND);
        let body: serde_json::Value =
            serde_json::from_slice(resp.body().as_bytes()).unwrap();
        assert_eq!(body["code"], "NOT_FOUND");
        assert_eq!(body["message"], "resource missing");
        assert_eq!(body["status"], 404);
    }

    #[test]
    fn response_error_with_tracing_includes_timestamp_and_request_id() {
        let resp = Response::error_with_tracing(
            HttpStatus::BAD_REQUEST,
            "BAD_REQ",
            "bad request",
            Some("req-123"),
        );
        assert_eq!(resp.status(), HttpStatus::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(resp.body().as_bytes()).unwrap();
        assert_eq!(body["code"], "BAD_REQ");
        assert_eq!(body["request_id"], "req-123");
        assert!(body["timestamp_ms"].as_u64().is_some());
    }

    #[test]
    fn response_error_with_tracing_no_request_id() {
        let resp = Response::error_with_tracing(
            HttpStatus::BAD_REQUEST,
            "BAD_REQ",
            "bad request",
            None,
        );
        let body: serde_json::Value =
            serde_json::from_slice(resp.body().as_bytes()).unwrap();
        assert!(body.get("request_id").is_none());
    }

    #[test]
    fn response_paginated_creates_correct_body() {
        let items = vec![1, 2, 3];
        let resp = Response::paginated(&items, 100, 0, 20).unwrap();
        assert_eq!(resp.status(), HttpStatus::OK);
        let body: serde_json::Value =
            serde_json::from_slice(resp.body().as_bytes()).unwrap();
        assert_eq!(body["total"], 100);
        assert_eq!(body["offset"], 0);
        assert_eq!(body["limit"], 20);
        assert_eq!(body["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn response_into_parts_splits_correctly() {
        let resp = Response::bytes(HttpStatus::OK, "body");
        let (status, headers, body) = resp.into_parts();
        assert_eq!(status, HttpStatus::OK);
        assert!(headers.is_empty());
        assert_eq!(body.as_bytes(), b"body");
    }

    #[test]
    fn response_set_body_replaces_body() {
        let mut resp = Response::empty(HttpStatus::OK);
        resp.set_body(Body::Bytes(vec![99]));
        assert_eq!(resp.body().as_bytes(), &[99]);
    }

    #[test]
    fn into_response_for_response_is_identity() {
        let resp = Response::empty(HttpStatus::OK);
        let result = resp.into_framework_response().unwrap();
        assert_eq!(result.status(), HttpStatus::OK);
    }

    #[test]
    fn into_response_for_unit_returns_204() {
        let result = ().into_framework_response().unwrap();
        assert_eq!(result.status(), HttpStatus::NO_CONTENT);
    }

    #[test]
    fn into_response_for_string_returns_ok() {
        let result = "hello".to_string().into_framework_response().unwrap();
        assert_eq!(result.status(), HttpStatus::OK);
        assert_eq!(result.body().as_bytes(), b"hello");
    }

    #[test]
    fn into_response_for_str_returns_ok() {
        let result = "static str".into_framework_response().unwrap();
        assert_eq!(result.status(), HttpStatus::OK);
        assert_eq!(result.body().as_bytes(), b"static str");
    }

    #[test]
    fn into_response_for_json_serializes() {
        let result = Json(42u32).into_framework_response().unwrap();
        assert_eq!(result.status(), HttpStatus::OK);
        assert_eq!(
            result.headers().get(http::header::CONTENT_TYPE).unwrap().to_str().unwrap(),
            "application/json"
        );
    }

    #[test]
    fn into_response_for_result_ok() {
        let result: Result<&'static str, ()> = Ok("ok");
        let resp = result.into_framework_response().unwrap();
        assert_eq!(resp.status(), HttpStatus::OK);
    }

    #[test]
    fn into_response_for_result_err() {
        let result: Result<(), &'static str> = Err("error");
        let resp = result.into_framework_response().unwrap();
        assert_eq!(resp.status(), HttpStatus::OK);
    }

    #[test]
    fn response_headers_mut_allows_modification() {
        let mut resp = Response::empty(HttpStatus::OK);
        resp.headers_mut()
            .insert("x-custom", "value".parse().unwrap());
        assert_eq!(
            resp.headers().get("x-custom").unwrap().to_str().unwrap(),
            "value"
        );
    }

    #[test]
    fn response_clone_produces_equal_copy() {
        let a = Response::bytes(HttpStatus::OK, vec![1, 2, 3]);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
