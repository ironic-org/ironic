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
