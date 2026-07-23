//! Redis client and reconnecting async connection registration.

use crate::{ProviderDefinition, integrations::IntegrationError};

use super::{IntegrationHealth, IntegrationHealthFuture};

/// The upstream Redis API used by this integration.
pub use ::redis as driver;

/// A cloneable Redis connection manager with automatic reconnection.
#[derive(Clone)]
pub struct RedisConnection(::redis::aio::ConnectionManager);

impl std::fmt::Debug for RedisConnection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RedisConnection")
            .finish_non_exhaustive()
    }
}

impl RedisConnection {
    /// Opens and verifies a Redis connection.
    ///
    /// # Errors
    ///
    /// Returns the upstream Redis error for invalid URLs or failed initial connections.
    pub async fn connect(url: &str) -> Result<Self, ::redis::RedisError> {
        let client = ::redis::Client::open(url)?;
        let connection = ::redis::aio::ConnectionManager::new(client).await?;
        Ok(Self(connection))
    }

    /// Returns a clone of the native reconnecting manager.
    #[must_use]
    pub fn manager(&self) -> ::redis::aio::ConnectionManager {
        self.0.clone()
    }
}

/// Registers this `RedisConnection` as a health indicator under the given name.
///
/// `RedisConnection` is cheap to clone (internally `Arc`-based connection manager).
pub fn register_health(connection: &RedisConnection, name: &'static str) {
    super::register_integration_health(name, connection.clone());
}

/// Registers a Redis connection manager as an Ironic singleton provider.
#[must_use]
pub fn provider(connection: RedisConnection) -> ProviderDefinition {
    ProviderDefinition::value(connection)
}

impl IntegrationHealth for RedisConnection {
    fn check_health(&self) -> IntegrationHealthFuture<'_> {
        Box::pin(async move {
            let mut connection = self.0.clone();
            ::redis::cmd("PING")
                .query_async::<String>(&mut connection)
                .await
                .map(|_| ())
                .map_err(|error| IntegrationError::new("REDIS", error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn redis_connect_fails_with_empty_url() {
        let result = RedisConnection::connect("").await;
        assert!(result.is_err());
    }

    #[test]
    fn redis_integration_error_display() {
        let err = IntegrationError::new("REDIS", "connection refused");
        assert_eq!(err.integration(), "REDIS");
        assert_eq!(err.to_string(), "IR_INTEGRATION_REDIS: connection refused");
    }
}
