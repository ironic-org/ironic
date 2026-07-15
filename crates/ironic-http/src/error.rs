#[cfg(feature = "backtrace")]
use std::sync::Arc;
use std::{error::Error, fmt};

use crate::{FrameworkResponse, HttpStatus, IntoFrameworkResponse};

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

impl IntoFrameworkResponse for HttpError {
    fn into_framework_response(self) -> Result<FrameworkResponse, HttpError> {
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
            return FrameworkResponse::json(self.status, &body);
        }
        Ok(FrameworkResponse::error(
            self.status,
            self.code,
            self.message,
        ))
    }
}
