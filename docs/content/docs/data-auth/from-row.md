---
title: Database Helpers (FromRow, SqlxErrorExt, AsyncModuleInit)
description: Eliminate sqlx boilerplate with compile-time-safe macros — FromRow for row mapping, SqlxErrorExt for error conversion, and AsyncModuleInit for async database startup.
---

# Database Developer Experience

## What you'll learn

- Derive `FromRow` instead of writing `r.get("column")` for every field
- Use `.map_db_err()` to turn sqlx errors into proper HTTP errors in one call
- Register `AsyncModuleInit` to run database connections and migrations during startup without `rt.block_on()` hacks

---

## `#[derive(FromRow)]` — Automatic Row Mapping

### The Problem

Without a derive, every sqlx query requires manual field-by-field row mapping:

```rust
let row = sqlx::query("SELECT id, title, content, slug, status, created_at FROM blog_posts WHERE id = $1")
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

let post = BlogPost {
    id: row.get("id"),
    title: row.get("title"),
    content: row.get("content"),
    slug: row.get("slug"),
    status: row.get("status"),
    created_at: row.get("created_at"),
};
```

Column name typos are runtime errors. In a 10-entity project, this pattern repeats hundreds of times.

### The Solution

Annotate your entity with `#[derive(FromRow)]`:

```rust
use ironic::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct BlogPost {
    #[sqlx(rename = "id")]
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub slug: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

Then use it with any sqlx fetch method:

```rust
let post: BlogPost = sqlx::query_as("SELECT * FROM blog_posts WHERE id = $1")
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_db_err("BLOG_POST", "FIND")?;
```

### Supported Attributes

| Attribute | Effect |
|-----------|--------|
| `#[sqlx(rename = "column_name")]` | Maps the field to a different column name |
| `#[sqlx(json)]` | Auto-deserializes a JSON/JSONB column via `serde_json::from_value` |
| `#[sqlx(default)]` | Uses `Default::default()` when the column is `NULL` or absent |

**Example with all attributes:**

```rust
#[derive(FromRow)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    #[sqlx(json)]
    pub tags: Vec<String>,
    #[sqlx(default)]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sqlx(rename = "author_id")]
    pub author: Uuid,
}
```

---

## `SqlxErrorExt` — Automatic Error Mapping

### The Problem

Every sqlx call needs explicit error conversion:

```rust
.map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
```

This produces inconsistent error codes and leaks raw database error messages.

### The Solution

Call `.map_db_err(entity, operation)?` on any `sqlx::Error`:

```rust
let post = sqlx::query_as::<_, BlogPost>("...")
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_db_err("BLOG_POST", "FIND")?;
```

**Error mapping rules:**

| sqlx Error | HTTP Status | Error Code |
|-----------|-------------|------------|
| `RowNotFound` | 404 | `DB_ROW_NOT_FOUND` |
| `PoolClosed` / `PoolTimedOut` | 503 | `DB_UNAVAILABLE` |
| Everything else | 500 | `DB_ERROR` |

The entity and operation names are included in the error message for diagnostics.

---

## `AsyncModuleInit` — Async Database Startup

### The Problem

`Module::definition()` is synchronous. Connecting to a database is async. The old pattern required `rt.block_on()` hacks:

```rust
impl Module for DatabaseModule {
    fn definition() -> ModuleDefinition {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool = rt.block_on(async {
            sqlx::PgPool::connect(&std::env::var("DATABASE_URL").unwrap()).await.unwrap()
        });
        ModuleDefinition::builder::<Self>()
            .provider(ProviderDefinition::value(pool))
            .build()
    }
}
```

### The Solution

Implement `AsyncModuleInit` on your module struct and register it with `#[module(async_init = [...])]`:

```rust
use ironic::AsyncModuleInit;

#[derive(Module)]
#[module(
    providers = [PgPool],
    exports = [PgPool],
    async_init = [DatabaseModule],
)]
pub struct DatabaseModule;

impl AsyncModuleInit for DatabaseModule {
    fn async_init<'a>(
        &'a self,
        container: &'a ironic_di::Container,
    ) -> ironic::LifecycleFuture<'a> {
        Box::pin(async move {
            let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            let pool = sqlx::PgPool::connect(&url).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok(())
        })
    }
}
```

`async_init()` runs during `Application::build()` — after the DI container is built but before any lifecycle hooks fire. You can resolve other providers from the container if needed.

---

## Putting It All Together

A complete database module with all three features:

```rust
use ironic::{FromRow, SqlxErrorExt, AsyncModuleInit, Module, SecretString};
use sqlx::PgPool;

#[derive(Module)]
#[module(
    providers = [PgPool],
    exports = [PgPool],
    async_init = [DatabaseModule],
)]
pub struct DatabaseModule;

impl AsyncModuleInit for DatabaseModule {
    fn async_init<'a>(
        &'a self,
        _container: &'a ironic_di::Container,
    ) -> ironic::LifecycleFuture<'a> {
        Box::pin(async move {
            let cfg = ConfigurationLoader::new()
                .file("ironic.toml")
                .environment("APP")
                .load::<AppConfig>()
                .expect("config must load");

            let pool = PgPool::connect(cfg.database.url.expose_secret()).await?;
            sqlx::migrate!("./migrations").run(&pool).await?;
            Ok(())
        })
    }
}

#[derive(FromRow)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    #[sqlx(json)]
    pub tags: Vec<String>,
}
```

---

## Feature Flags

| Feature | Required |
|---------|----------|
| `FromRow` derive | `sqlx-postgres`, `sqlx-mysql`, `sqlx-sqlite`, or `sqlx` |
| `SqlxErrorExt` | `sqlx` |
| `AsyncModuleInit` | Always available |
