use ironic_http::{HeaderMap, HttpStatus, Response};
use serde::{Serialize, de::DeserializeOwned};

/// An in-process response with typed accessors and focused assertions.
///
/// Wraps an [`ironic_http::Response`] and provides convenience methods for
/// common test assertions such as status code, header values, and JSON body
/// comparison.
pub struct TestResponse {
    response: Response,
}

impl TestResponse {
    pub(crate) const fn new(response: Response) -> Self {
        Self { response }
    }

    /// Returns the HTTP status.
    #[must_use]
    pub const fn status(&self) -> HttpStatus {
        self.response.status()
    }

    /// Returns response headers.
    #[must_use]
    pub const fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }

    /// Returns the raw response body.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        self.response.body().as_bytes()
    }

    /// Deserializes the JSON response body.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when the body is not valid JSON for `T`.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(self.body())
    }

    /// Asserts the numeric response status.
    ///
    /// # Panics
    ///
    /// Panics when the actual status differs from `expected`.
    pub fn assert_status(&self, expected: u16) {
        assert_eq!(
            self.status().as_u16(),
            expected,
            "unexpected response status; body: {}",
            String::from_utf8_lossy(self.body())
        );
    }

    /// Asserts one response header value.
    ///
    /// # Panics
    ///
    /// Panics when the header is missing or its value differs from `expected`.
    pub fn assert_header(&self, name: &str, expected: &str) {
        let actual = self
            .headers()
            .get(name)
            .unwrap_or_else(|| panic!("response header `{name}` is missing"));
        assert_eq!(actual, expected, "unexpected value for header `{name}`");
    }

    /// Asserts structural JSON equality.
    ///
    /// # Panics
    ///
    /// Panics when either value cannot be represented as JSON or the values differ.
    pub fn assert_json<T: Serialize + ?Sized>(&self, expected: &T) {
        let actual: serde_json::Value = self
            .json()
            .unwrap_or_else(|error| panic!("response body is not valid JSON: {error}"));
        let expected = serde_json::to_value(expected)
            .unwrap_or_else(|error| panic!("expected value cannot be serialized: {error}"));
        assert_eq!(actual, expected, "unexpected JSON response body");
    }

    /// Asserts the framework error code in a structured error response.
    ///
    /// # Panics
    ///
    /// Panics when the body is not a structured error or its code differs from `expected_code`.
    pub fn assert_error(&self, expected_code: &str) {
        let body: serde_json::Value = self
            .json()
            .unwrap_or_else(|error| panic!("response body is not a structured error: {error}"));
        assert_eq!(
            body["code"], expected_code,
            "unexpected framework error code"
        );
    }

    /// Consumes the wrapper and returns the framework response.
    #[must_use]
    pub fn into_inner(self) -> Response {
        self.response
    }
}

#[cfg(test)]
mod tests {
    use ironic_http::{HeaderMap, HeaderValue, HttpStatus, Response};

    use super::TestResponse;

    fn ok_response() -> TestResponse {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers.insert("x-request-id", HeaderValue::from_static("abc"));
        let response =
            Response::json(HttpStatus::OK, &serde_json::json!({"key": "value"})).unwrap();
        TestResponse::new(response)
    }

    fn error_response() -> TestResponse {
        let response = Response::error(
            HttpStatus::NOT_FOUND,
            "RF_RESOURCE_NOT_FOUND",
            "The resource was not found",
        );
        TestResponse::new(response)
    }

    #[test]
    fn status_returns_http_status() {
        let resp = ok_response();
        assert_eq!(resp.status(), HttpStatus::OK);
    }

    #[test]
    fn headers_returns_header_map() {
        let resp = ok_response();
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn body_returns_bytes() {
        let resp = ok_response();
        assert!(!resp.body().is_empty());
    }

    #[test]
    fn json_deserializes_body() {
        let resp = ok_response();
        let value: serde_json::Value = resp.json().unwrap();
        assert_eq!(value["key"], "value");
    }

    #[test]
    fn json_returns_error_on_empty_body() {
        let response = Response::empty(HttpStatus::OK);
        let resp = TestResponse::new(response);
        assert!(resp.json::<serde_json::Value>().is_err());
    }

    #[test]
    fn assert_status_passes() {
        ok_response().assert_status(200);
    }

    #[test]
    #[should_panic(expected = "unexpected response status")]
    fn assert_status_panics_on_mismatch() {
        ok_response().assert_status(404);
    }

    #[test]
    fn assert_header_passes() {
        ok_response().assert_header("content-type", "application/json");
    }

    #[test]
    #[should_panic(expected = "response header `x-missing` is missing")]
    fn assert_header_panics_on_missing() {
        ok_response().assert_header("x-missing", "value");
    }

    #[test]
    fn assert_json_passes() {
        ok_response().assert_json(&serde_json::json!({"key": "value"}));
    }

    #[test]
    #[should_panic(expected = "unexpected JSON response body")]
    fn assert_json_panics_on_mismatch() {
        ok_response().assert_json(&serde_json::json!({"key": "wrong"}));
    }

    #[test]
    fn assert_error_passes() {
        error_response().assert_error("RF_RESOURCE_NOT_FOUND");
    }

    #[test]
    #[should_panic(expected = "unexpected framework error code")]
    fn assert_error_panics_on_code_mismatch() {
        error_response().assert_error("RF_WRONG_CODE");
    }

    #[test]
    #[should_panic(expected = "unexpected framework error code")]
    fn assert_error_panics_on_non_error_body() {
        ok_response().assert_error("ANY");
    }

    #[test]
    fn into_inner_consumes_wrapper() {
        let resp = ok_response();
        let inner = resp.into_inner();
        assert_eq!(inner.status(), HttpStatus::OK);
    }

    #[test]
    fn empty_response_has_empty_body() {
        let response = Response::empty(HttpStatus::NO_CONTENT);
        let resp = TestResponse::new(response);
        assert_eq!(resp.status(), HttpStatus::NO_CONTENT);
        assert!(resp.body().is_empty());
    }
}
