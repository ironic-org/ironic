#[cfg(feature = "backtrace")]
use std::sync::Arc;
use std::{error::Error, fmt};

use crate::{HttpStatus, IntoResponse, Response};

/// A safe, structured HTTP request or handler failure.
#[derive(Clone, Debug)]
pub struct HttpError {
    status: HttpStatus,
    code: &'static str,
    message: String,
    #[cfg(feature = "backtrace")]
    backtrace: Option<Arc<std::backtrace::Backtrace>>,
}

impl PartialEq for HttpError {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status && self.code == other.code && self.message == other.message
    }
}

impl Eq for HttpError {}

impl HttpError {
    /// Creates a structured HTTP error.
    ///
    /// `status` is the HTTP response status code, `code` is a stable
    /// machine-readable error identifier (e.g. `"RF_HTTP_NOT_FOUND"`),
    /// and `message` is a human-readable description safe for clients.
    #[must_use]
    pub fn new(status: HttpStatus, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            #[cfg(feature = "backtrace")]
            backtrace: None,
        }
    }

    /// Creates a malformed-request error.
    #[must_use]
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(HttpStatus::BAD_REQUEST, code, message)
    }

    /// Creates a not-found error.
    #[must_use]
    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(HttpStatus::NOT_FOUND, code, message)
    }

    /// Creates an authentication-required error.
    #[must_use]
    pub fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(HttpStatus::UNAUTHORIZED, code, message)
    }

    /// Creates a forbidden error.
    #[must_use]
    pub fn forbidden(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(HttpStatus::FORBIDDEN, code, message)
    }

    /// Creates a validation error.
    #[must_use]
    pub fn unprocessable_entity(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(HttpStatus::UNPROCESSABLE_ENTITY, code, message)
    }

    /// Creates a redacted internal error.
    ///
    /// When the `backtrace` feature is enabled, captures a backtrace at the
    /// call site automatically.
    #[must_use]
    pub fn internal(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: HttpStatus::INTERNAL_SERVER_ERROR,
            code,
            message: message.into(),
            #[cfg(feature = "backtrace")]
            backtrace: Some(Arc::new(std::backtrace::Backtrace::capture())),
        }
    }

    /// Attaches a backtrace to this error.
    ///
    /// Has no effect when the `backtrace` feature is disabled.
    #[must_use]
    #[cfg(feature = "backtrace")]
    pub fn with_backtrace(mut self) -> Self {
        if self.backtrace.is_none() {
            self.backtrace = Some(Arc::new(std::backtrace::Backtrace::capture()));
        }
        self
    }

    /// Returns the response status.
    #[must_use]
    pub const fn status(&self) -> HttpStatus {
        self.status
    }

    /// Returns the stable public error code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Returns the safe public message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl Error for HttpError {}

impl IntoResponse for HttpError {
    fn into_framework_response(self) -> Result<Response, HttpError> {
        #[cfg(feature = "backtrace")]
        if cfg!(debug_assertions)
            && let Some(backtrace) = self.backtrace
        {
            #[derive(serde::Serialize)]
            struct ErrorBody {
                status: u16,
                code: String,
                message: String,
                backtrace: String,
            }

            let body = ErrorBody {
                status: self.status.as_u16(),
                code: self.code.to_owned(),
                message: self.message,
                backtrace: backtrace.to_string(),
            };
            return Response::json(self.status, &body);
        }
        Ok(Response::error(self.status, self.code, self.message))
    }
}

#[cfg(test)]
mod tests {
    use crate::{HttpError, HttpStatus, IntoResponse};

    #[test]
    fn new_creates_error_with_correct_fields() {
        let err = HttpError::new(HttpStatus::BAD_REQUEST, "TEST_ERR", "test message");
        assert_eq!(err.status(), HttpStatus::BAD_REQUEST);
        assert_eq!(err.code(), "TEST_ERR");
        assert_eq!(err.message(), "test message");
    }

    #[test]
    fn bad_request_uses_400() {
        let err = HttpError::bad_request("BAD", "bad request");
        assert_eq!(err.status(), HttpStatus::BAD_REQUEST);
    }

    #[test]
    fn not_found_uses_404() {
        let err = HttpError::not_found("NF", "not found");
        assert_eq!(err.status(), HttpStatus::NOT_FOUND);
    }

    #[test]
    fn unauthorized_uses_401() {
        let err = HttpError::unauthorized("UNAUTH", "unauthorized");
        assert_eq!(err.status(), HttpStatus::UNAUTHORIZED);
    }

    #[test]
    fn forbidden_uses_403() {
        let err = HttpError::forbidden("FORBID", "forbidden");
        assert_eq!(err.status(), HttpStatus::FORBIDDEN);
    }

    #[test]
    fn unprocessable_entity_uses_422() {
        let err = HttpError::unprocessable_entity("VALIDATION", "validation failed");
        assert_eq!(err.status(), HttpStatus::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn internal_uses_500() {
        let err = HttpError::internal("INT_ERR", "internal error");
        assert_eq!(err.status(), HttpStatus::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn display_format_includes_code_and_message() {
        let err = HttpError::new(HttpStatus::BAD_REQUEST, "DISPLAY_TEST", "something broke");
        assert_eq!(format!("{err}"), "DISPLAY_TEST: something broke");
    }

    #[test]
    fn partial_eq_ignores_backtrace() {
        let a = HttpError::new(HttpStatus::BAD_REQUEST, "EQ_TEST", "eq");
        let b = HttpError::new(HttpStatus::BAD_REQUEST, "EQ_TEST", "eq");
        assert_eq!(a, b);
    }

    #[test]
    fn partial_eq_different_status_not_equal() {
        let a = HttpError::new(HttpStatus::BAD_REQUEST, "CODE", "msg");
        let b = HttpError::new(HttpStatus::NOT_FOUND, "CODE", "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn partial_eq_different_code_not_equal() {
        let a = HttpError::new(HttpStatus::BAD_REQUEST, "CODE_A", "msg");
        let b = HttpError::new(HttpStatus::BAD_REQUEST, "CODE_B", "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn into_framework_response_produces_ok() {
        let err = HttpError::new(HttpStatus::IM_A_TEAPOT, "TEAPOT", "I am a teapot");
        let result = err.into_framework_response();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status(), HttpStatus::IM_A_TEAPOT);
    }

    #[test]
    fn error_trait_is_implemented() {
        use std::error::Error;
        let err = HttpError::new(HttpStatus::BAD_REQUEST, "TRAIT", "error trait");
        let source = (&err as &dyn Error).source();
        assert!(source.is_none());
    }
}
