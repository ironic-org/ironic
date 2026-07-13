//! `MongoDB` client and database registration with connectivity checks.

use crate::{ProviderDefinition, integrations::IntegrationError};

use super::{IntegrationHealth, IntegrationHealthFuture};

/// The upstream `MongoDB` API used by this integration.
pub use ::mongodb as driver;

/// A named `MongoDB` database and its shared native client.
#[derive(Clone, Debug)]
pub struct MongoDatabase {
    client: ::mongodb::Client,
    database: ::mongodb::Database,
}

impl MongoDatabase {
    /// Connects to a `MongoDB` deployment and selects `database_name`.
    ///
    /// # Errors
    ///
    /// Returns the upstream `MongoDB` error when the URI is invalid or the client cannot initialize.
    pub async fn connect(uri: &str, database_name: &str) -> Result<Self, ::mongodb::error::Error> {
        let client = ::mongodb::Client::with_uri_str(uri).await?;
        let database = client.database(database_name);
        Ok(Self { client, database })
    }

    /// Returns the native `MongoDB` client.
    #[must_use]
    pub const fn client(&self) -> &::mongodb::Client {
        &self.client
    }

    /// Returns the selected native `MongoDB` database.
    #[must_use]
    pub const fn database(&self) -> &::mongodb::Database {
        &self.database
    }
}

/// Registers a named `MongoDB` database as an Ironic singleton provider.
#[must_use]
pub fn provider(database: MongoDatabase) -> ProviderDefinition {
    ProviderDefinition::value(database)
}

impl IntegrationHealth for MongoDatabase {
    fn check_health(&self) -> IntegrationHealthFuture<'_> {
        Box::pin(async move {
            self.database
                .run_command(::mongodb::bson::doc! { "ping": 1 })
                .await
                .map(|_| ())
                .map_err(|error| IntegrationError::new("MONGODB", error))
        })
    }
}
