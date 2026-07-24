//! GraphQL integration for Ironic — resolver, query, mutation, and subscription
//! decorators that integrate with the DI container and module system.
//!
//! Requires the `graphql` feature.

pub use async_graphql as driver;

use async_graphql::{Schema, SchemaBuilder, EmptySubscription};

/// A builder for constructing a merged GraphQL schema from resolvers.
pub struct GraphqlSchemaBuilder<Q, M, S> {
    schema_builder: SchemaBuilder<Q, M, S>,
}

impl<Q, M, S> GraphqlSchemaBuilder<Q, M, S>
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    /// Creates a new schema builder with the given root types.
    pub fn new(query: Q, mutation: M, subscription: S) -> Self {
        Self {
            schema_builder: Schema::build(query, mutation, subscription),
        }
    }

    /// Builds the schema.
    pub fn finish(self) -> Schema<Q, M, S> {
        self.schema_builder.finish()
    }
}

/// Convenience type alias for a minimal schema with no subscriptions.
pub type QueryOnlySchema<Q> = Schema<Q, async_graphql::EmptyMutation, EmptySubscription>;
