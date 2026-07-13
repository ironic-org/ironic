use rustframe_http::{FrameworkResponse, HeaderMap, HttpStatus};
use serde::{Serialize, de::DeserializeOwned};

/// An in-process response with typed accessors and focused assertions.
pub struct TestResponse {
    response: FrameworkResponse,
}

impl TestResponse {
    pub(crate) const fn new(response: FrameworkResponse) -> Self {
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
    pub fn into_inner(self) -> FrameworkResponse {
        self.response
    }
}
