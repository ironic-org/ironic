//! Diesel `r2d2` pool construction, registration, and health checks.

use crate::{ProviderDefinition, integrations::IntegrationError};

use super::{IntegrationHealth, IntegrationHealthFuture};

/// The upstream Diesel API used by this integration.
pub use ::diesel as driver;

/// A Diesel-managed `r2d2` connection pool.
pub type DieselPool<C> = ::diesel::r2d2::Pool<::diesel::r2d2::ConnectionManager<C>>;

/// Builds a Diesel pool which validates connections when checked out.
///
/// # Errors
///
/// Returns the upstream pool error when its initial connection cannot be established.
pub fn connect<C>(
    database_url: impl Into<String>,
) -> Result<DieselPool<C>, ::diesel::r2d2::PoolError>
where
    C: ::diesel::r2d2::R2D2Connection + Send + 'static,
{
    let manager = ::diesel::r2d2::ConnectionManager::<C>::new(database_url);
    ::diesel::r2d2::Pool::builder()
        .test_on_check_out(true)
        .build(manager)
}

/// Registers an existing Diesel pool as an Ironic singleton provider.
#[must_use]
pub fn provider<C>(pool: DieselPool<C>) -> ProviderDefinition
where
    C: ::diesel::r2d2::R2D2Connection + Send + 'static,
{
    ProviderDefinition::value(pool)
}

impl<C> IntegrationHealth for DieselPool<C>
where
    C: ::diesel::r2d2::R2D2Connection + Send + 'static,
{
    fn check_health(&self) -> IntegrationHealthFuture<'_> {
        Box::pin(async move {
            let pool = self.clone();
            tokio::task::spawn_blocking(move || pool.get().map(|_| ()))
                .await
                .map_err(|error| IntegrationError::new("DIESEL", error))?
                .map_err(|error| IntegrationError::new("DIESEL", error))
        })
    }
}
