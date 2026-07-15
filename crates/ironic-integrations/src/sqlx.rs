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

/// Registers this pool as a health indicator under the given name.
///
/// The pool is cheap to clone (internally `Arc`-based), so the registered
/// indicator will remain valid even if the original handle is dropped.
pub fn register_health<DB>(pool: &::sqlx::Pool<DB>, name: &'static str)
where
    DB: ::sqlx::Database,
{
    super::register_integration_health(name, pool.clone());
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
