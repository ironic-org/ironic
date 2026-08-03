---
title: Diesel
description: Schema-first ORM with Diesel — schema definitions, models, repositories, CLI migrations, and async pool setup.
---

# Diesel


## Feature flag

```toml
ironic = { features = ["diesel"] }
```

## Schema definition

```rust
// src/schema.rs
diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        created_at -> Timestamptz,
    }
}
```

## Model

```rust
// src/models/user.rs
use diesel::prelude::*;
use crate::schema::users;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

## Repository

```rust
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Injectable)]
pub struct UserRepository {
    pool: Arc<PgPool>,  // diesel_async connection pool
}

impl UserRepository {
    pub async fn find_by_id(&self, id: i32) -> Result<User, HttpError> {
        use crate::schema::users::dsl::*;

        let mut conn = self.pool
            .get()
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        users
            .filter(id.eq(id))
            .first::<User>(&mut conn)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }

    pub async fn list(&self) -> Result<Vec<User>, HttpError> {
        use crate::schema::users::dsl::*;

        let mut conn = self.pool
            .get()
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        users
            .order(id.asc())
            .load::<User>(&mut conn)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }
}
```

## CLI

```bash
# Install diesel_cli
cargo install diesel_cli --no-default-features --features postgres

# Setup
diesel setup

# Create migration
diesel migration generate create_users

# Run migrations
diesel migration run
```

---

## Manual connection setup


When you need full control over pool configuration, build the connection manually and register it as a provider:

### Diesel (async, via `diesel_async`)

```rust
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

let url = std::env::var("DATABASE_URL").unwrap();
let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&url);

let pool = bb8::Pool::builder()
    .max_size(20)
    .connection_timeout(Duration::from_secs(5))
    .max_lifetime(Some(Duration::from_secs(1800)))
    .idle_timeout(Some(Duration::from_secs(600)))
    .build(config)
    .await?;

let pool: Arc<bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>> = Arc::new(pool);
```
