//! `SeaORM` connection registration and health checks.

use crate::{ProviderDefinition, integrations::IntegrationError};

use super::{IntegrationHealth, IntegrationHealthFuture};

/// The upstream `SeaORM` API used by this integration.
pub use ::sea_orm as driver;

/// Connects `SeaORM` using its native connection options.
///
/// # Errors
///
/// Returns the upstream `SeaORM` database error when the connection cannot be established.
pub async fn connect(
    options: impl Into<::sea_orm::ConnectOptions>,
) -> Result<::sea_orm::DatabaseConnection, ::sea_orm::DbErr> {
    ::sea_orm::Database::connect(options).await
}

/// Registers this connection as a health indicator under the given name.
///
/// The connection is cheap to clone (internally `Arc`-based).
pub fn register_health(connection: &::sea_orm::DatabaseConnection, name: &'static str) {
    super::register_integration_health(name, connection.clone());
}

/// Registers an existing `SeaORM` connection as an Ironic singleton provider.
#[must_use]
pub fn provider(connection: ::sea_orm::DatabaseConnection) -> ProviderDefinition {
    ProviderDefinition::value(connection)
}

impl IntegrationHealth for ::sea_orm::DatabaseConnection {
    fn check_health(&self) -> IntegrationHealthFuture<'_> {
        Box::pin(async move {
            self.ping()
                .await
                .map_err(|error| IntegrationError::new("SEAORM", error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn seaorm_connect_fails_with_empty_url() {
        let result = connect(String::new()).await;
        assert!(result.is_err());
    }

    #[test]
    fn seaorm_integration_error_display() {
        let err = IntegrationError::new("SEAORM", "connection timeout");
        assert_eq!(err.integration(), "SEAORM");
        assert_eq!(err.to_string(), "IR_INTEGRATION_SEAORM: connection timeout");
    }
}
