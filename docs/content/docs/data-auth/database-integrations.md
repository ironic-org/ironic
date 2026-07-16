---
title: Database Integrations
description: Connect Ironic to PostgreSQL, MySQL, SQLite, MongoDB, and Redis — with connection pooling, migrations, health checks, and injection-ready clients.
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
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono"] }
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
| PostgreSQL (SQLx) | `sqlx-postgres` | `sqlx` | `Arc<PgPool>` |
| MySQL (SQLx) | `sqlx-mysql` | `sqlx` | `Arc<MySqlPool>` |
| SQLite (SQLx) | `sqlx-sqlite` | `sqlx` | `Arc<SqlitePool>` |
| SeaORM (Postgres) | `seaorm-postgres` | `sea-orm` | `Arc<DatabaseConnection>` |
| Diesel (Postgres/MySQL/SQLite) | `diesel` | `diesel` | `Arc<PgConnection>` |
| MongoDB | `mongodb` | `mongodb` | `Arc<Client>` |
| Redis | `redis` | `redis` | `Arc<ConnectionManager>` |

---

## SQLx — PostgreSQL

### Feature flag

```toml
ironic = { features = ["sqlx-postgres"] }
```

### Connection URL

```
postgres://user:password@localhost:5432/mydb
```

Config in `ironic.toml`:

```toml
[settings]
database_url = "postgres://user:password@localhost:5432/mydb"
```

Or set `DATABASE_URL` environment variable.

### Repository pattern

```rust
use ironic::integrations::sqlx::{PgPool, Postgres};
use ironic::prelude::*;

#[derive(Injectable)]
pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: i64) -> Result<User, HttpError> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
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

    pub async fn create(&self, name: &str, email: &str) -> Result<User, HttpError> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        )
        .bind(name)
        .bind(email)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }

    pub async fn delete(&self, id: i64) -> Result<(), HttpError> {
        let rows = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
            .rows_affected();

        if rows == 0 {
            return Err(HttpError::not_found("USER_NOT_FOUND", format!("User {id} not found")));
        }
        Ok(())
    }
}
```

### Transactions

```rust
pub async fn transfer_points(
    &self,
    from_id: i64,
    to_id: i64,
    amount: i64,
) -> Result<(), HttpError> {
    let mut tx = self.pool
        .begin()
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

    let result = sqlx::query("UPDATE balances SET points = points - $1 WHERE user_id = $2")
        .bind(amount).bind(from_id)
        .execute(&mut *tx)
        .await;

    if result.map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?.rows_affected() == 0 {
        tx.rollback().await.ok();
        return Err(HttpError::not_found("USER_NOT_FOUND", "Sender not found"));
    }

    sqlx::query("UPDATE balances SET points = points + $1 WHERE user_id = $2")
        .bind(amount).bind(to_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
}
```

### Migrations

```bash
# Install sqlx-cli
cargo install sqlx-cli

# Create migration
sqlx migrate add create_users_table

# Run migrations
sqlx migrate run
```

Migration file (`migrations/20240101000000_create_users_table.sql`):

```sql
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

Run migrations at startup:

```rust
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!();  // from migrations/ dir

pub async fn run_migrations(pool: &PgPool) -> Result<(), HttpError> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| HttpError::internal("MIGRATION_FAILED", e.to_string()))
}
```

---

## SQLx — MySQL

### Feature flag

```toml
ironic = { features = ["sqlx-mysql"] }
```

### Connection URL

```
mysql://user:password@localhost:3306/mydb
```

### Usage

```rust
use ironic::integrations::sqlx::{MySqlPool, MySql};

#[derive(Injectable)]
pub struct ProductRepository {
    pool: Arc<MySqlPool>,
}

impl ProductRepository {
    pub async fn find_by_id(&self, id: i64) -> Result<Product, HttpError> {
        sqlx::query_as::<_, Product>("SELECT * FROM products WHERE id = ?")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
            .ok_or_else(|| HttpError::not_found("NOT_FOUND", format!("Product {id} not found")))
    }
}
```

---

## SQLx — SQLite

### Feature flag

```toml
ironic = { features = ["sqlx-sqlite"] }
```

### Connection URL

```
sqlite:app.db?mode=rwc
```

### Usage

```rust
use ironic::integrations::sqlx::{SqlitePool, Sqlite};

#[derive(Injectable)]
pub struct SettingsRepository {
    pool: Arc<SqlitePool>,
}

impl SettingsRepository {
    pub async fn get(&self, key: &str) -> Result<Option<String>, HttpError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM settings WHERE key = ?",
        )
        .bind(key)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        Ok(row.map(|r| r.0))
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), HttpError> {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
        )
        .bind(key)
        .bind(value)
        .execute(&*self.pool)
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        Ok(())
    }
}
```

---

## SeaORM

### Feature flag

```toml
ironic = { features = ["seaorm-postgres"] }
```

### Connection URL

```
postgres://user:password@localhost:5432/mydb
```

### Entity definition

```rust
// src/entities/user.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

### Repository

```rust
use sea_orm::*;

#[derive(Injectable)]
pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: i32) -> Result<user::Model, HttpError> {
        user::Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
            .ok_or_else(|| HttpError::not_found("NOT_FOUND", format!("User {id} not found")))
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<user::Model, HttpError> {
        let active = user::ActiveModel {
            name: Set(name.to_owned()),
            email: Set(email.to_owned()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        active
            .insert(&*self.db)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }

    pub async fn update_email(&self, id: i32, email: &str) -> Result<(), HttpError> {
        let mut user: user::ActiveModel = self
            .find_by_id(id)
            .await?
            .into();

        user.email = Set(email.to_owned());

        user.update(&*self.db)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        Ok(())
    }
}
```

### CLI

```bash
# Install sea-orm-cli
cargo install sea-orm-cli

# Generate entities from existing database
sea-orm-cli generate entity -o src/entities
```

---

## Diesel

### Feature flag

```toml
ironic = { features = ["diesel"] }
```

### Schema definition

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

### Model

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

### Repository

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

### CLI

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

## MongoDB

### Feature flag

```toml
ironic = { features = ["mongodb"] }
```

### Connection URL

```
mongodb://user:password@localhost:27017/mydb
```

### Repository

```rust
use mongodb::{Client, Collection, bson::{doc, Document, oid::ObjectId}};

#[derive(Injectable)]
pub class UserService {
    db: Arc<Client>,
}

impl UserService {
    fn users(&self) -> Collection<Document> {
        self.db.database("mydb").collection("users")
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Document, HttpError> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| HttpError::bad_request("INVALID_ID", "Invalid ObjectId"))?;

        self.users()
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
            .ok_or_else(|| HttpError::not_found("NOT_FOUND", format!("User {id} not found")))
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<Document, HttpError> {
        let doc = doc! {
            "name": name,
            "email": email,
            "created_at": chrono::Utc::now().to_rfc3339(),
        };

        let result = self.users()
            .insert_one(doc)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        self.find_by_id(&result.inserted_id.as_object_id().unwrap().to_hex())
            .await
    }

    pub async fn list(&self) -> Result<Vec<Document>, HttpError> {
        let mut cursor = self.users()
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .limit(50)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        let mut results = Vec::new();
        while cursor.advance().await.map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))? {
            results.push(cursor.deserialize_current().unwrap());
        }

        Ok(results)
    }
}
```

---

## Redis

### Feature flag

```toml
ironic = { features = ["redis"] }
```

### Connection URL

```
redis://user:password@localhost:6379
```

### Service

```rust
use redis::AsyncCommands;

#[derive(Injectable)]
pub class CacheService {
    conn: Arc<ConnectionManager>,
}

impl CacheService {
    pub async fn get(&self, key: &str) -> Result<Option<String>, HttpError> {
        let mut conn = self.conn.clone();
        conn
            .get(format!("cache:{key}"))
            .await
            .map_err(|e| HttpError::internal("CACHE_ERROR", e.to_string()))
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), HttpError> {
        let mut conn = self.conn.clone();
        conn
            .set_ex(format!("cache:{key}"), value, ttl_secs as usize)
            .await
            .map_err(|e| HttpError::internal("CACHE_ERROR", e.to_string()))
    }

    pub async fn delete(&self, key: &str) -> Result<(), HttpError> {
        let mut conn = self.conn.clone();
        conn
            .del(format!("cache:{key}"))
            .await
            .map_err(|e| HttpError::internal("CACHE_ERROR", e.to_string()))?;
        Ok(())
    }

    pub async fn increment(&self, key: &str) -> Result<i64, HttpError> {
        let mut conn = self.conn.clone();
        conn
            .incr(format!("rate:{key}"), 1)
            .await
            .map_err(|e| HttpError::internal("CACHE_ERROR", e.to_string()))
    }
}
```

### Redis as a rate limiter

```rust
pub async fn check_rate_limit(
    &self,
    user_id: &str,
    max_requests: i64,
    window_secs: u64,
) -> Result<(), HttpError> {
    let mut conn = self.conn.clone();
    let key = format!("ratelimit:{user_id}");

    let count: i64 = conn
        .incr(&key)
        .await
        .map_err(|e| HttpError::internal("CACHE_ERROR", e.to_string()))?;

    if count == 1 {
        let _: () = conn
            .expire(&key, window_secs as usize)
            .await
            .ok();
    }

    if count > max_requests {
        return Err(HttpError::too_many_requests("RATE_LIMITED", "Too many requests"));
    }

    Ok(())
}
```

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

## Manual connection setup

When you need full control over pool configuration, build the connection manually and register it as a provider:

### SQLx (PostgreSQL)

```rust
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn build_pg_pool() -> Result<PgPool, HttpError> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| HttpError::internal("CONFIG_ERROR", "DATABASE_URL not set"))?;

    PgPoolOptions::new()
        .max_connections(20)            // pool size
        .max_lifetime(Duration::from_secs(1800))  // recycle connections every 30 min
        .idle_timeout(Duration::from_secs(600))   // close idle after 10 min
        .acquire_timeout(Duration::from_secs(5))  // wait max 5s for a connection
        .connect(&url)
        .await
        .map_err(|e| HttpError::internal("DB_CONNECT_FAILED", e.to_string()))
}
```

Then register the pool as a provider in your module:

```rust
use ironic::prelude::*;

#[derive(Module)]
#[module(providers = [build_pg_pool])]
struct AppModule;

#[provider]
async fn build_pg_pool() -> Result<Arc<PgPool>, HttpError> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

    Ok(Arc::new(pool))
}
```

### SQLx (MySQL)

```rust
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

let pool = MySqlPoolOptions::new()
    .max_connections(15)
    .acquire_timeout(Duration::from_secs(3))
    .connect("mysql://user:password@localhost:3306/mydb")
    .await?;
```

### SQLx (SQLite)

```rust
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

let pool = SqlitePoolOptions::new()
    .max_connections(5)        // SQLite is single-writer, keep pool small
    .connect("sqlite:app.db?mode=rwc")
    .await?;
```

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

### MongoDB

```rust
use mongodb::{Client, options::ClientOptions};

let mut options = ClientOptions::parse("mongodb://user:password@localhost:27017/mydb")
    .await?;

options.max_pool_size = Some(20);
options.min_pool_size = Some(5);
options.connect_timeout = Some(Duration::from_secs(10));
options.server_selection_timeout = Some(Duration::from_secs(5));
options.app_name = Some("ironic-app".to_string());

let client = Client::with_options(options)?;
let client: Arc<Client> = Arc::new(client);
```

### Redis

```rust
use redis::{ConnectionManager, RedisConnectionInfo};

let client = redis::Client::open("redis://user:password@localhost:6379")
    .map_err(|e| HttpError::internal("REDIS_ERROR", e.to_string()))?;

let manager = ConnectionManager::new(client)
    .await
    .map_err(|e| HttpError::internal("REDIS_ERROR", e.to_string()))?;

let manager: Arc<ConnectionManager> = Arc::new(manager);
```

### Full example with custom config and health check

```rust
use std::sync::Arc;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[provider]
async fn provide_pg_pool() -> Result<Arc<PgPool>, HttpError> {
    let url = dotenvy::var("DATABASE_URL")
        .map_err(|_| HttpError::internal("CONFIG", "DATABASE_URL missing"))?;

    let pool = PgPoolOptions::new()
        .max_connections(
            dotenvy::var("DB_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        )
        .acquire_timeout(Duration::from_secs(
            dotenvy::var("DB_ACQUIRE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
        ))
        .connect(&url)
        .await
        .map_err(|e| HttpError::internal("DB_CONNECT", e.to_string()))?;

    // Verify connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| HttpError::internal("DB_PING", e.to_string()))?;

    tracing::info!("PostgreSQL pool ready (max: {})", pool.size());
    Ok(Arc::new(pool))
}
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

## Testing with test databases

### SQLx in-memory SQLite

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_user() {
        let pool = test_pool().await;
        let repo = UserRepository { pool: Arc::new(pool) };

        let user = repo.create("Alice", "alice@example.com").await.unwrap();
        assert_eq!(user.name, "Alice");
    }
}
```

### Testcontainers for PostgreSQL

```rust
#[cfg(test)]
mod tests {
    use testcontainers::{runners::AsyncRunner, ContainerAsync};
    use testcontainers_modules::postgres::Postgres;

    async fn setup_postgres() -> (ContainerAsync<Postgres>, PgPool) {
        let container = Postgres::default()
            .start()
            .await
            .unwrap();

        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            container.get_host_port_ipv4(5432).await.unwrap()
        );

        let pool = PgPool::connect(connection_string).await.unwrap();

        sqlx::query("CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, email TEXT)")
            .execute(&pool)
            .await
            .unwrap();

        (container, pool)
    }
}
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
