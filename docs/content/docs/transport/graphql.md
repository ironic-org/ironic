---
title: GraphQL Transport
description: GraphQL API support with schema-first or code-first approach, queries, mutations, and subscriptions.
---

# GraphQL Transport

GraphQL support lets you expose a query language API alongside your REST endpoints. Ironic uses `async-graphql` under the hood, integrated with the DI container for resolver resolution.

## Enabling GraphQL

Enable the `graphql` feature:

```toml
[dependencies]
ironic = { version = "1.0", features = ["graphql"] }
```

## Schema Provider

Register a GraphQL schema provider as an injectable service:

```rust
use ironic::*;
use async_graphql::*;

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn users(&self, ctx: &Context<'_>) -> Vec<User> {
        // Resolver can access DI container
        let repo = ctx.data::<Arc<UserRepository>>()?;
        repo.find_all().await
    }
}

#[injectable]
fn schema_provider() -> Schema<QueryRoot, EmptyMutation, EmptySubscription> {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .finish()
}
```

Then wire it into a controller:

```rust
#[controller("/graphql")]
struct GraphqlController {
    schema: Arc<Schema<QueryRoot, EmptyMutation, EmptySubscription>>,
}

#[routes]
impl GraphqlController {
    #[post("/")]
    async fn execute(&self, body: JsonBody<GraphQLRequest>) -> Json<GraphQLResponse> {
        let response = body.0.execute(&self.schema).await;
        Json(response)
    }
}
```

## What's Supported

- **Queries**: Read-only data fetching with filtering, pagination, sorting
- **Mutations**: Data modification with validation
- **Subscriptions**: Real-time updates via WebSocket (requires `realtime` feature)
- **DI Integration**: Resolvers can inject dependencies from the container
- **Error Handling**: GraphQL errors mapped from framework exceptions

## Configuration

```rust
use ironic::distributed::graphql::GraphQLConfig;

let config = GraphQLConfig {
    max_depth: 32,
    max_complexity: 1000,
    enable_federation: false,
    // ...
};
```

## Roadmap

- **Federation support** for microservice GraphQL gateways
- **Automatic schema generation** from entity definitions
- **Batch query optimization** (DataLoader integration)
- **Persisted queries** for production optimization
