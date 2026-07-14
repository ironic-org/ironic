---
title: Database Integrations
description: Connect Ironic to PostgreSQL, MySQL, SQLite, MongoDB, and Redis — with built-in connection management.
---

# Database Integrations

## What you'll learn

- Choose the right database for your app
- Set up connection pools that Ironic manages for you
- Use SQLx, SeaORM, Diesel, MongoDB, or Redis

## The big picture

```
Your App ──► Ironic (connection pool) ──► PostgreSQL / MySQL / SQLite / MongoDB / Redis
                  │
                  ├── Auto-creates connection pool
                  ├── Health checks included
                  └── Injection-ready (Arc<Pool>)
```

## Available integrations

| Database | Feature flag | Best for |
|----------|-------------|----------|
| **SQLx** (Postgres) | `sqlx-postgres` | Raw SQL, migrations, compile-time checked queries |
| **SQLx** (MySQL) | `sqlx-mysql` | Raw SQL with MySQL |
| **SQLx** (SQLite) | `sqlx-sqlite` | Embedded, testing, small apps |
| **SeaORM** (Postgres) | `seaorm-postgres` | ORM with relations, Active Record pattern |
| **Diesel** | `diesel` | Type-safe query builder, schema-first |
| **MongoDB** | `mongodb` | Document store, flexible schema |
| **Redis** | `redis` | Caching, sessions, pub/sub, rate limiting |

> **Recommendation:** Start with `sqlx-postgres` — it's the simplest and most well-tested.

## SQLx example (PostgreSQL)

### Cargo.toml

```toml
ironic = { features = ["sqlx-postgres"] }
```

### Your service

```rust
use ironic::integrations::sqlx::{PgPool, Postgres};
use ironic::prelude::*;

#[derive(Injectable)]
pub struct UserRepository {
    pool: std::sync::Arc<PgPool>,    // ← Connection pool injected automatically
}

impl UserRepository {
    pub async fn find_by_id(&self, id: u64) -> Result<User, HttpError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id as i64)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
            .ok_or_else(|| HttpError::not_found("USER_NOT_FOUND", format!("User {id} not found")))
    }

    pub async fn list(&self) -> Result<Vec<User>, HttpError> {
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }
}
```

### Configuration in `ironic.toml`

```toml
[settings]
database_url = "postgres://user:password@localhost:5432/mydb"
```

> The connection pool is created automatically and injected wherever you use `Arc<PgPool>`.

## SeaORM example

```rust
use ironic::integrations::seaorm::DatabaseConnection;
use sea_orm::*;

#[derive(Injectable)]
pub struct UserRepository {
    db: std::sync::Arc<DatabaseConnection>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: u64) -> Result<Option<user::Model>, DbErr> {
        user::Entity::find_by_id(id as i64).one(&*self.db).await
    }
}
```

## Health checks

Each integration comes with built-in health checks:

```rust
// The framework automatically:
// 1. Pings the database on startup
// 2. Reports connection status in /health
// 3. Reconnects if the connection drops
```

## Try it yourself

1. Add `sqlx-postgres` feature flag
2. Set `database_url` in `ironic.toml`
3. Create a `UserRepository` with `Arc<PgPool>`
4. Write a `find_all` method that returns all users
5. Call it from a controller

## What you learned

- [x] Choose from SQLx, SeaORM, Diesel, MongoDB, and Redis
- [x] Connection pools are auto-created and injected
- [x] Use `Arc<PgPool>` (or equivalent) in your services
- [x] Health checks are included automatically
