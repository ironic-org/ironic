use serde::Serialize;

use crate::{HeaderMap, HeaderValue, HttpError, HttpStatus};

/// An owned transport-neutral response body.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum FrameworkBody {
    /// No response body.
    #[default]
    Empty,
    /// A complete in-memory response body.
    Bytes(Vec<u8>),
}

impl FrameworkBody {
    /// Returns the body as bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Empty => &[],
            Self::Bytes(bytes) => bytes,
        }
    }
}

/// An owned transport-neutral HTTP response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameworkResponse {
    status: HttpStatus,
    headers: HeaderMap,
    body: FrameworkBody,
}

impl FrameworkResponse {
    /// Creates an empty response with `status`.
    #[must_use]
    pub fn empty(status: HttpStatus) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: FrameworkBody::Empty,
        }
    }

    /// Creates a byte response.
    #[must_use]
    pub fn bytes(status: HttpStatus, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: FrameworkBody::Bytes(body.into()),
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
    pub const fn body(&self) -> &FrameworkBody {
        &self.body
    }

    /// Splits the response into transport-owned parts.
    #[must_use]
    pub fn into_parts(self) -> (HttpStatus, HeaderMap, FrameworkBody) {
        (self.status, self.headers, self.body)
    }

    /// Replaces the response body.
    pub fn set_body(&mut self, body: FrameworkBody) {
        self.body = body;
    }
}

/// Converts a handler result into a framework response.
pub trait IntoFrameworkResponse {
    /// Performs response conversion.
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] when conversion or serialization fails.
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError>;
}

impl IntoFrameworkResponse for FrameworkResponse {
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
        Ok(self)
    }
}

impl IntoFrameworkResponse for () {
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
        Ok(FrameworkResponse::empty(HttpStatus::NO_CONTENT))
    }
}

impl IntoFrameworkResponse for String {
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
        Ok(FrameworkResponse::bytes(HttpStatus::OK, self))
    }
}

impl IntoFrameworkResponse for &'static str {
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
        self.to_owned().into_framework_response()
    }
}

/// Marks a value for JSON response serialization.
#[derive(Clone, Copy, Debug)]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoFrameworkResponse for Json<T> {
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
        FrameworkResponse::json(HttpStatus::OK, &self.0)
    }
}

impl<T, E> IntoFrameworkResponse for Result<T, E>
where
    T: IntoFrameworkResponse,
    E: IntoFrameworkResponse,
{
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
        match self {
            Ok(value) => value.into_framework_response(),
            Err(error) => error.into_framework_response(),
        }
    }
}
