---
title: Database integrations
description: Register SQLx, SeaORM, Diesel, MongoDB, and Redis clients with Ironic dependency injection.
---

# Database integrations

Database support is opt-in. Ironic wraps each library's native pool or client instead of replacing
its query API, migrations, transactions, or configuration types.

## Features

```toml
[dependencies]
ironic = { version = "0.1", features = ["sqlx-postgres"] }
```

Available features are:

| Integration | Ironic feature |
| --- | --- |
| SQLx core | `sqlx` |
| SQLx PostgreSQL | `sqlx-postgres` |
| SQLx MySQL | `sqlx-mysql` |
| SQLx SQLite | `sqlx-sqlite` |
| SeaORM core | `seaorm` |
| SeaORM PostgreSQL | `seaorm-postgres` |
| SeaORM MySQL | `seaorm-mysql` |
| SeaORM SQLite | `seaorm-sqlite` |
| Diesel pooling | `diesel` |
| MongoDB | `mongodb` |
| Redis | `redis` |

`database` enables every backend-neutral integration API. Select a SQLx or SeaORM driver feature
separately. Diesel backend selection remains on the application's direct Diesel dependency because
its PostgreSQL and MySQL features can require native system libraries.

## SQLx

```rust
use ironic::{ProviderDefinition, integrations::sqlx};
use sqlx::driver::postgres::PgPoolOptions;

# async fn example() -> Result<(), sqlx::driver::Error> {
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect("postgres://localhost/app")
    .await?;
let provider: ProviderDefinition = sqlx::provider(pool);
# Ok(())
# }
```

Use `integrations::sqlx::migrate` with a native `sqlx::migrate::Migrator`. Pools implement
`IntegrationHealth`, which acquires and returns a connection without running application queries.

## SeaORM

```rust
use ironic::integrations::seaorm;

# async fn example() -> Result<(), seaorm::driver::DbErr> {
let connection = seaorm::connect("sqlite::memory:").await?;
let provider = seaorm::provider(connection);
# let _ = provider;
# Ok(())
# }
```

The registered value is the native `sea_orm::DatabaseConnection`; repositories can inject and use
it directly. Health checks delegate to SeaORM's native `ping` operation.

## Diesel

Select a backend on Diesel itself and enable Ironic's backend-neutral pool integration:

```toml
[dependencies]
ironic = { version = "0.1", features = ["diesel"] }
diesel = { version = "2.2", features = ["postgres", "r2d2"] }
```

```rust
use diesel::PgConnection;
use ironic::integrations::diesel;

# fn example() -> Result<(), diesel::r2d2::PoolError> {
let pool = diesel::connect::<PgConnection>("postgres://localhost/app")?;
let provider = diesel::provider(pool);
# let _ = provider;
# Ok(())
# }
```

Diesel connections are blocking. Ironic's health check uses `tokio::task::spawn_blocking` so a pool
checkout does not block the asynchronous executor.

## MongoDB

```rust
use ironic::integrations::mongodb::{self, MongoDatabase};

# async fn example() -> Result<(), mongodb::driver::error::Error> {
let database = MongoDatabase::connect("mongodb://localhost:27017", "app").await?;
let users = database.database().collection::<mongodb::driver::bson::Document>("users");
let provider = mongodb::provider(database);
# let _ = (users, provider);
# Ok(())
# }
```

The wrapper retains both the native client and selected database. Its health check sends MongoDB's
standard `ping` command.

## Redis

```rust
use ironic::integrations::redis::{self, RedisConnection};

# async fn example() -> Result<(), redis::driver::RedisError> {
let connection = RedisConnection::connect("redis://127.0.0.1/").await?;
let manager = connection.manager();
let provider = redis::provider(connection);
# let _ = (manager, provider);
# Ok(())
# }
```

`RedisConnection` uses the native reconnecting async connection manager. It is cloneable for
concurrent commands, and its health check sends `PING`.

## Health checks

Every integration handle implements the common `IntegrationHealth` contract:

```text
use ironic::integrations::IntegrationHealth;

database.check_health().await?;
```

Connection strings can contain credentials. Do not include them in logs or public error responses;
load them through `SecretString` and expose them only while constructing the native client.
