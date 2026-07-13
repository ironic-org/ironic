//! Optional integrations with database drivers and object-relational mappers.

use std::{future::Future, pin::Pin};

/// A boxed health-check future returned by an integration.
pub type IntegrationHealthFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), IntegrationError>> + Send + 'a>>;

/// A connectivity or configuration error reported by an integration.
#[derive(Debug, thiserror::Error)]
#[error("IR_INTEGRATION_{integration}: {message}")]
pub struct IntegrationError {
    integration: &'static str,
    message: String,
}

impl IntegrationError {
    /// Creates an integration error while retaining only a safe display message.
    #[must_use]
    pub fn new(integration: &'static str, error: impl std::fmt::Display) -> Self {
        Self {
            integration,
            message: error.to_string(),
        }
    }

    /// Returns the integration identifier used in diagnostics.
    #[must_use]
    pub const fn integration(&self) -> &'static str {
        self.integration
    }
}

/// A uniform connectivity check implemented by database integration handles.
pub trait IntegrationHealth: Send + Sync {
    /// Checks whether the configured integration can serve work.
    fn check_health(&self) -> IntegrationHealthFuture<'_>;
}

#[cfg(feature = "diesel")]
pub mod diesel;
#[cfg(feature = "mongodb")]
pub mod mongodb;
#[cfg(feature = "redis")]
pub mod redis;
#[cfg(feature = "seaorm")]
pub mod seaorm;
#[cfg(feature = "sqlx")]
pub mod sqlx;
