---
title: SQLx (PostgreSQL, MySQL, SQLite)
description: Use SQLx with PostgreSQL, MySQL, and SQLite in Ironic — repository pattern, transactions, migrations, and manual pool setup.
---

# SQLx

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
