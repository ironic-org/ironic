//! GraphQL integration for Ironic — resolver, query, mutation, and subscription
//! decorators that integrate with the DI container and module system.
//!
//! Requires the `graphql` feature.

pub use async_graphql as driver;

use async_graphql::{EmptySubscription, Schema, SchemaBuilder};

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

    /// Enables Apollo Federation support for this schema.
    #[must_use]
    pub fn enable_federation(mut self) -> Self {
        self.schema_builder = self.schema_builder.enable_federation();
        self
    }

    /// Registers a custom directive.
    #[must_use]
    pub fn directive(mut self, directive: impl async_graphql::CustomDirectiveFactory) -> Self {
        self.schema_builder = self.schema_builder.directive(directive);
        self
    }

    /// Builds the schema.
    pub fn finish(self) -> Schema<Q, M, S> {
        self.schema_builder.finish()
    }
}

/// Convenience type alias for a minimal schema with no subscriptions.
pub type QueryOnlySchema<Q> = Schema<Q, async_graphql::EmptyMutation, EmptySubscription>;
