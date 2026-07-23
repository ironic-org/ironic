//! GraphQL schema integration using async-graphql.

/// The upstream async-graphql API.
pub use ::async_graphql as driver;

use crate::ProviderDefinition;

/// Registers an executable GraphQL schema as a singleton provider.
///
/// # Examples
///
/// ```ignore
/// use ironic::distributed::graphql::schema_provider;
/// use async_graphql::{EmptyMutation, EmptySubscription, Schema};
///
/// struct QueryRoot;
/// #[async_graphql::Object]
/// impl QueryRoot {
///     async fn version(&self) -> &str { "1.0" }
/// }
///
/// let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
/// let provider = schema_provider(schema);
/// ```
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

#[cfg(test)]
mod tests {
    #[test]
    fn async_graphql_driver_re_exports() {
        let _ = super::driver::EmptyMutation;
        let _ = super::driver::EmptySubscription;
    }
}
