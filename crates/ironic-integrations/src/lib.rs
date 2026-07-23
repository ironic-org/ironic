//! Optional integrations with database drivers and object-relational mappers.

use std::{future::Future, pin::Pin, sync::Arc};

use ironic_core::{HealthIndicator, HealthStatus};

/// A boxed health-check future returned by an integration.
pub type IntegrationHealthFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), IntegrationError>> + Send + 'a>>;

/// A connectivity or configuration error reported by an integration.
///
/// # Errors
///
/// Constructed by [`IntegrationError::new`] whenever an integration
/// endpoint is unreachable or returns an unexpected response.
///
/// # Panics
///
/// Never panics on its own.
#[derive(Debug, thiserror::Error)]
#[error("IR_INTEGRATION_{integration}: {message}")]
pub struct IntegrationError {
    integration: &'static str,
    message: String,
}

impl IntegrationError {
    /// Creates an integration error while retaining only a safe display
    /// message.
    ///
    /// # Errors
    ///
    /// This is a pure constructor and does not perform any I/O.
    ///
    /// # Panics
    ///
    /// Never panics.
    #[must_use]
    pub fn new(integration: &'static str, error: impl std::fmt::Display) -> Self {
        Self {
            integration,
            message: error.to_string(),
        }
    }

    /// Returns the integration identifier used in diagnostics.
    ///
    /// # Errors
    ///
    /// This is a pure accessor and never fails.
    ///
    /// # Panics
    ///
    /// Never panics.
    #[must_use]
    pub const fn integration(&self) -> &'static str {
        self.integration
    }
}

/// A uniform connectivity check implemented by database integration handles.
///
/// # Errors
///
/// [`check_health`](IntegrationHealth::check_health) returns an error when the
/// integration cannot reach its backing service.
///
/// # Panics
///
/// Implementations should not panic under normal operation.
pub trait IntegrationHealth: Send + Sync {
    /// Checks whether the configured integration can serve work.
    fn check_health(&self) -> IntegrationHealthFuture<'_>;
}

// ---------------------------------------------------------------------------
// HealthIndicator wrapper for any IntegrationHealth implementation
// ---------------------------------------------------------------------------

struct HealthIndicatorWrapper<T: IntegrationHealth> {
    name: &'static str,
    inner: T,
}

impl<T: IntegrationHealth + 'static> HealthIndicator for HealthIndicatorWrapper<T> {
    fn name(&self) -> &str {
        self.name
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async move {
            match self.inner.check_health().await {
                Ok(()) => HealthStatus::Ok,
                Err(e) => HealthStatus::Unhealthy {
                    error: e.to_string(),
                },
            }
        })
    }
}

/// Registers an [`IntegrationHealth`] implementor as a [`HealthIndicator`]
/// so it appears on the `GET /health` composite endpoint.
///
/// Call this from each integration module after creating the connection/pool.
pub fn register_integration_health<T: IntegrationHealth + 'static>(name: &'static str, inner: T) {
    ironic_core::register_health_indicator(Arc::new(HealthIndicatorWrapper { name, inner }));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_error_new_and_getter() {
        let err = IntegrationError::new("TEST", "something failed");
        assert_eq!(err.integration(), "TEST");
        assert_eq!(err.to_string(), "IR_INTEGRATION_TEST: something failed");
    }

    #[test]
    fn integration_error_display() {
        let err = IntegrationError::new("MYMOD", 42);
        assert_eq!(err.to_string(), "IR_INTEGRATION_MYMOD: 42");
    }

    #[allow(deprecated)]
    #[tokio::test]
    async fn health_indicator_wrapper_maps_ok() {
        struct OkIntegration;
        impl IntegrationHealth for OkIntegration {
            fn check_health(&self) -> IntegrationHealthFuture<'_> {
                Box::pin(async { Ok(()) })
            }
        }

        let wrapper = HealthIndicatorWrapper {
            name: "OK_INT",
            inner: OkIntegration,
        };
        let status = wrapper.check().await;
        assert!(matches!(status, HealthStatus::Ok));
    }

    #[allow(deprecated)]
    #[tokio::test]
    async fn health_indicator_wrapper_maps_error() {
        struct BrokenIntegration;
        impl IntegrationHealth for BrokenIntegration {
            fn check_health(&self) -> IntegrationHealthFuture<'_> {
                Box::pin(async { Err(IntegrationError::new("BRKN", "down")) })
            }
        }

        let wrapper = HealthIndicatorWrapper {
            name: "BRKN",
            inner: BrokenIntegration,
        };
        let status = wrapper.check().await;
        assert!(matches!(status, HealthStatus::Unhealthy { .. }));
    }
}
