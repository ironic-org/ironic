---
title: Database Integrations
description: Set up and connect Ironic to PostgreSQL, MySQL, SQLite, MongoDB, and Redis — with pooling, health checks, and injection-ready clients.
---

# Database Integrations

## What you'll learn

- Choose the right database for your app
- Set up connection pools that Ironic manages for you
- Use SQLx, SeaORM, Diesel, MongoDB, or Redis with proper patterns
- Run migrations, handle transactions, and test with real databases
- Add database support to a newly generated `ironic new` project

---

## Setting up a database in a generated project

When you run `ironic new my-project`, the scaffold uses in-memory storage by default with no database dependencies. Here's how to add one.

### 1. Add database features to `Cargo.toml`

```toml
# Before (generated defaults)
ironic = { features = ["security", "compression", "metrics", "validation"] }

# After — PostgreSQL via SQLx
ironic = { features = ["security", "compression", "metrics", "validation", "sqlx-postgres"] }
sqlx = { version = "0.9", features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono"] }
```

Other options:

| Database | `ironic` feature | Additional crate |
|---|---|---|
| MySQL (SQLx) | `sqlx-mysql` | `sqlx = { features = ["runtime-tokio", "mysql"] }` |
| SQLite (SQLx) | `sqlx-sqlite` | `sqlx = { features = ["runtime-tokio", "sqlite"] }` |
| SeaORM | `seaorm-postgres` | `sea-orm = { features = ["sqlx-postgres", "runtime-tokio"] }` |
| Diesel | `diesel` | `diesel = { features = ["postgres"] }` + `diesel-async = { features = ["bb8"] }` |
| MongoDB | `mongodb` | `mongodb = { version = "3", features = ["tokio-runtime"] }` |
| Redis | `redis` | `redis = { features = ["tokio-comp", "connection-manager"] }` |

### 2. Set `DATABASE_URL` in `.env`

```bash
# Uncomment and set your connection string
DATABASE_URL=postgres://user:password@localhost:5432/my_database
```

The Docker Compose file in your generated project already includes Postgres and Redis containers — just make sure the credentials match.

### 3. Create a provider for the connection pool

Add this to `src/main.rs` or a new `src/database.rs`:

```rust
use std::sync::Arc;
use sqlx::postgres::{PgPool, PgPoolOptions};
use ironic::prelude::*;

#[provider]
async fn provide_pool() -> Result<Arc<PgPool>, HttpError> {
    let url = dotenvy::var("DATABASE_URL")
        .map_err(|_| HttpError::internal("CONFIG", "DATABASE_URL must be set"))?;

    let pool = PgPoolOptions::new()
        .max_connections(
            dotenvy::var("DB_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        )
        .connect(&url)
        .await
        .map_err(|e| HttpError::internal("DB_CONNECT", e.to_string()))?;

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| HttpError::internal("MIGRATION", e.to_string()))?;

    Ok(Arc::new(pool))
}
```

### 4. Register the provider in your module

```rust
// src/app.rs
#[derive(Module)]
#[module(providers = [provide_pool, /* ... other providers */])]
struct AppModule;
```

### 5. Replace in-memory storage with database calls

The generated `ExampleService` uses `Mutex<HashMap<u64, Example>>`. Change it to inject the pool:

```rust
use sqlx::PgPool;

#[derive(Injectable)]
pub struct ExampleRepository {
    pool: Arc<PgPool>,
}

impl ExampleRepository {
    pub async fn list(&self) -> Result<Vec<Example>, HttpError> {
        sqlx::query_as::<_, Example>("SELECT * FROM examples ORDER BY id")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }

    pub async fn create(&self, data: CreateExampleDto) -> Result<Example, HttpError> {
        sqlx::query_as::<_, Example>(
            "INSERT INTO examples (name, description) VALUES ($1, $2) RETURNING *",
        )
        .bind(&data.name)
        .bind(&data.description)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }
}
```

### 6. Create your first migration

```bash
# Install sqlx-cli
cargo install sqlx-cli

# Create the migrations directory
sqlx migrate add create_examples_table
```

Edit the generated file in `migrations/`:

```sql
CREATE TABLE examples (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### 7. Start the database and run

```bash
# Start Postgres via Docker Compose (included in generated project)
docker compose up -d postgres

# Start your app
ironic start
```

The pool is auto-injected into any service or controller that declares `Arc<PgPool>`.

---

## Available integrations

| Database | Feature flag | Crate | Pool type |
|---|---|---|---|
| [PostgreSQL (SQLx)](./db-sqlx) | `sqlx-postgres` | `sqlx` | `Arc<PgPool>` |
| [MySQL (SQLx)](./db-sqlx) | `sqlx-mysql` | `sqlx` | `Arc<MySqlPool>` |
| [SQLite (SQLx)](./db-sqlx) | `sqlx-sqlite` | `sqlx` | `Arc<SqlitePool>` |
| [SeaORM](./db-seaorm) | `seaorm-postgres` | `sea-orm` | `Arc<DatabaseConnection>` |
| [Diesel](./db-diesel) | `diesel` | `diesel` | `Arc<PgConnection>` |
| [MongoDB](./db-mongodb) | `mongodb` | `mongodb` | `Arc<Client>` |
| [Redis](./db-redis) | `redis` | `redis` | `Arc<ConnectionManager>` |

> Testing recipes (in-memory SQLite, Testcontainers PostgreSQL) live in [Testing with test databases](./db-testing).

---

## Pool configuration

| Parameter | Env variable | Default | Description |
|---|---|---|---|
| `database_url` | `DATABASE_URL` | — | Connection string |
| `db_pool_size` | `DB_POOL_SIZE` | 10 | Maximum connections in pool |
| `db_max_lifetime` | `DB_MAX_LIFETIME` | 30 min | Max connection lifetime |
| `db_idle_timeout` | `DB_IDLE_TIMEOUT` | 10 min | Close idle connections |
| `db_acquire_timeout` | `DB_ACQUIRE_TIMEOUT` | 5 sec | Timeout when pool is exhausted |

```toml
[settings]
database_url = "postgres://user:password@localhost:5432/mydb"
db_pool_size = 20
db_max_lifetime = 3600
db_idle_timeout = 600
db_acquire_timeout = 3
```

---

## Container-style DI

All database clients are registered via `Arc` so they can be injected into any service or controller:

```rust
#[controller("/api/users")]
pub struct UserController {
    repo: Arc<UserRepository>,       // injected pool dependency
    cache: Arc<CacheService>,        // injected Redis dependency
}

#[get("/:id")]
async fn get_user(&self, id: Path<i64>) -> Result<Json<User>, HttpError> {
    let user = self.repo.find_by_id(*id).await?;
    Ok(Json(user))
}
```

---

## Health checks

Each integration registers a health check that pings the database:

```rust
// Automatic — nothing to configure
// Reports at GET /health
//
// {
//   "status": "ok",
//   "checks": {
//     "database": { "status": "up", "latency_ms": 2 },
//     "redis":     { "status": "up", "latency_ms": 1 }
//   }
// }
```

---

## Common mistakes

| Mistake | Fix |
|---|---|
| `database_url` not set | Add to `ironic.toml` or set `DATABASE_URL` env var |
| Pool exhausted under load | Increase `db_pool_size` |
| Connection refused | Verify database is running and port is correct |
| SSL error | Add `?sslmode=require` or `?sslmode=disable` to connection URL |
| Migration not found | Run `sqlx migrate run` or embed with `sqlx::migrate!()` |
| MongoDB `ObjectId` parse fail | Validate the ID string format before querying |
| Redis connection timeout | Check Redis is reachable and `redis://` URL format is correct |

---

## What you learned

- [x] Connect to PostgreSQL, MySQL, SQLite via SQLx with full CRUD examples
- [x] Use SeaORM with entity generation and Active Record pattern
- [x] Use Diesel with schema-first approach and CLI migrations
- [x] Use MongoDB for document storage with cursor pagination
- [x] Use Redis for caching, rate limiting, and session-like patterns
- [x] Configure pool size, timeouts, and connection lifetime
- [x] Test with in-memory SQLite and Testcontainers PostgreSQL
- [x] Transactions, migrations, and error handling for each database
