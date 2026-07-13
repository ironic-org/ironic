use std::{error::Error, fmt};

use crate::{FrameworkResponse, HttpStatus, IntoFrameworkResponse};

/// A safe, structured HTTP request or handler failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpError {
    status: HttpStatus,
    code: &'static str,
    message: String,
}

impl HttpError {
    /// Creates a structured HTTP error.
    #[must_use]
    pub fn new(status: HttpStatus, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
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
    #[must_use]
    pub fn internal(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(HttpStatus::INTERNAL_SERVER_ERROR, code, message)
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

impl IntoFrameworkResponse for HttpError {
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
        Ok(FrameworkResponse::error(
            self.status,
            self.code,
            self.message,
        ))
    }
}
