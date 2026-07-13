//! GraphQL schema integration using async-graphql.

/// The upstream async-graphql API.
pub use ::async_graphql as driver;

use crate::ProviderDefinition;

/// Registers an executable GraphQL schema as a singleton provider.
#[must_use]
pub fn schema_provider<Query, Mutation, Subscription>(
    schema: driver::Schema<Query, Mutation, Subscription>,
) -> ProviderDefinition
where
    Query: driver::ObjectType + Send + Sync + 'static,
    Mutation: driver::ObjectType + Send + Sync + 'static,
    Subscription: driver::SubscriptionType + Send + Sync + 'static,
{
    ProviderDefinition::value(schema)
}
