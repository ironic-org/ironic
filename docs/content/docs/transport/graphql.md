---
title: GraphQL
description: GraphQL API with resolver decorators, mutations, subscriptions, and Federation
---

# GraphQL

Ironic provides deep GraphQL integration through `async-graphql` with DI-powered
resolver decorators and schema building.

## Enabling

```toml
[dependencies]
ironic = { features = ["graphql"] }
```

## Resolvers

Use `#[resolver]` to create DI-injectable GraphQL resolvers:

```rust
use ironic::{resolver, gql_query, mutation, subscription};

#[resolver]
struct UserResolver {
    user_service: UserService,
}

#[gql_query]
async fn users(&self) -> Vec<User> {
    self.user_service.list().await
}

#[mutation]
async fn create_user(&self, name: String) -> User {
    self.user_service.create(name).await
}

#[subscription]
async fn user_added(&self) -> impl futures_util::Stream<Item = User> {
    self.user_service.subscribe()
}
```

## Schema Builder

```rust
use ironic::graphql_integration::{GraphqlSchemaBuilder, QueryOnlySchema};
use async_graphql::{EmptyMutation, EmptySubscription};

struct Query;
#[Object]
impl Query {
    async fn hello(&self) -> &str { "world" }
}

let schema: QueryOnlySchema<Query> = GraphqlSchemaBuilder::new(
    Query, EmptyMutation, EmptySubscription
).finish();
```

## Federation

```rust
let schema = GraphqlSchemaBuilder::new(Query, EmptyMutation, EmptySubscription)
    .enable_federation()
    .finish();
```

## Full async-graphql Access

All `async-graphql` types are available via `ironic::graphql_integration::driver`:

```rust
use ironic::graphql_integration::driver;
// driver::Scalar, driver::Interface, driver::Union, driver::CustomDirective, etc.
```
