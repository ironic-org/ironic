//! `SQLx` pool registration, migrations, and health checks.

use crate::{ProviderDefinition, integrations::IntegrationError};

use super::{IntegrationHealth, IntegrationHealthFuture};

/// The upstream `SQLx` API used by this integration.
pub use ::sqlx as driver;

/// Registers an existing `SQLx` pool as an Ironic singleton provider.
#[must_use]
pub fn provider<DB>(pool: ::sqlx::Pool<DB>) -> ProviderDefinition
where
    DB: ::sqlx::Database,
{
    ProviderDefinition::value(pool)
}

/// Runs a prebuilt `SQLx` migrator against a pool.
///
/// # Errors
///
/// Returns the upstream migration error without discarding its diagnostic context.
pub async fn migrate<DB>(
    migrator: &::sqlx::migrate::Migrator,
    pool: &::sqlx::Pool<DB>,
) -> Result<(), ::sqlx::migrate::MigrateError>
where
    DB: ::sqlx::Database,
    DB::Connection: ::sqlx::migrate::Migrate,
{
    migrator.run(pool).await
}

impl<DB> IntegrationHealth for ::sqlx::Pool<DB>
where
    DB: ::sqlx::Database,
{
    fn check_health(&self) -> IntegrationHealthFuture<'_> {
        Box::pin(async move {
            self.acquire()
                .await
                .map(|_| ())
                .map_err(|error| IntegrationError::new("SQLX", error))
        })
    }
}
