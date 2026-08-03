---
title: Testing with Test Databases
description: Test database-backed code with in-memory SQLite and Testcontainers for PostgreSQL.
---

# Testing with Test Databases

## Testing with test databases

## SQLx in-memory SQLite

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

## Testcontainers for PostgreSQL

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
