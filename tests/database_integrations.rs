//! Feature-level contracts for optional database integrations.

#[cfg(any(feature = "sqlx-sqlite", feature = "seaorm"))]
use ironic::ProviderKey;
use ironic::integrations::IntegrationError;

#[test]
fn integration_errors_identify_the_failing_driver() {
    let error = IntegrationError::new("TEST", "unavailable");
    assert_eq!(error.integration(), "TEST");
    assert!(error.to_string().contains("unavailable"));
}

#[cfg(feature = "sqlx-sqlite")]
#[tokio::test]
async fn sqlx_pools_register_and_report_health() {
    use ironic::integrations::{IntegrationHealth, sqlx};

    let pool = ::sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_lazy("sqlite::memory:")
        .unwrap();
    assert_eq!(
        sqlx::provider(pool.clone()).key(),
        ProviderKey::of::<::sqlx::SqlitePool>()
    );
    pool.check_health().await.unwrap();
}

#[cfg(feature = "seaorm")]
#[test]
fn seaorm_connections_register_as_singletons() {
    let connection = ::sea_orm::DatabaseConnection::Disconnected;
    assert_eq!(
        ironic::integrations::seaorm::provider(connection).key(),
        ProviderKey::of::<::sea_orm::DatabaseConnection>()
    );
}

#[cfg(feature = "mongodb")]
#[tokio::test]
async fn mongodb_rejects_invalid_connection_strings() {
    assert!(
        ironic::integrations::mongodb::MongoDatabase::connect("://invalid", "test")
            .await
            .is_err()
    );
}

#[cfg(feature = "redis")]
#[tokio::test]
async fn redis_rejects_invalid_connection_strings() {
    assert!(
        ironic::integrations::redis::RedisConnection::connect("://invalid")
            .await
            .is_err()
    );
}

#[cfg(feature = "diesel")]
#[allow(dead_code)]
fn diesel_pool_api_is_backend_generic<C>()
where
    C: ::diesel::r2d2::R2D2Connection + Send + 'static,
{
    let _: fn(ironic::integrations::diesel::DieselPool<C>) -> ironic::ProviderDefinition =
        ironic::integrations::diesel::provider::<C>;
}
