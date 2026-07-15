---
title: Sessions
description: Manage user sessions with an in-memory store for development and Redis-backed persistence for production.
---

# Sessions

## What you'll learn

- Enable sessions with the `sessions` feature
- Choose between in-memory and Redis-backed session stores
- Store and retrieve session data in route handlers
- Real-world patterns: authentication, flash messages, shopping cart
- Best practices for session TTL and security

---

## Enabling sessions

```toml
# In-memory only (development)
ironic = { features = ["sessions"] }

# With Redis persistence (production)
ironic = { features = ["sessions", "redis"] }
```

## Quick start

```rust
use std::sync::Arc;
use ironic::auth::sessions::{InMemorySessionStore, SessionStore, Session};

// Create a store
let store = Arc::new(InMemorySessionStore::new());

// Create a new session
let mut session = store.create().await?;

// Store data
session.insert("user_id", 42u64)?;
session.insert("role", "admin")?;

// Persist
store.save(&session).await?;

// Retrieve later
let loaded = store.get(&session.id()).await?;
let user_id: u64 = loaded.get("user_id")?;
```

## InMemorySessionStore (development)

Keeps sessions in a `HashMap` behind `Arc<Mutex<...>>`.  All sessions are lost
when the process restarts.

```rust
use ironic::auth::sessions::{InMemorySessionStore, SessionStore};

let store = Arc::new(InMemorySessionStore::new());

// Optional: configure session TTL
let store = Arc::new(InMemorySessionStore::with_ttl(Duration::from_secs(3600)));
```

**Use for:** local development, unit tests, single-replica demos.

## RedisSessionStore (production)

Persists sessions in Redis using `SETEX` / `GET` / `DEL`.  Session data is
serialized as JSON under the key `ironic:session:{id}`.

```rust
use ironic::auth::sessions::{RedisSessionStore, RedisSessionConfig};
use redis::aio::ConnectionManager;

let client = redis::Client::open("redis://127.0.0.1:6379")?;
let connection_manager = ConnectionManager::new(client).await?;

let store = RedisSessionStore::new(connection_manager)
    .with_ttl(Duration::from_secs(86400));  // 24 hours
```

| Config | Default | Description |
|--------|---------|-------------|
| `session_ttl` | `86400` | Session TTL in seconds (24h). Renewed on every `save()`. |

### Redis URL formats

```
redis://127.0.0.1:6379
redis://user:password@127.0.0.1:6379/1
rediss://127.0.0.1:6379           (TLS)
redis://127.0.0.1:6379?timeout=2s (connection timeout)
```

### Key namespace

All session keys are prefixed with `ironic:session:` so they coexist with other
data in the same Redis instance:

```
ironic:session:a1b2c3d4-e5f6-7890-abcd-ef1234567890
ironic:session:b2c3d4e5-f6a7-8901-bcde-f12345678901
```

## Real-world pattern: authentication session

```rust
use std::sync::Arc;
use ironic::auth::sessions::{SessionStore, Session};
use ironic::Inject;

#[derive(Clone)]
struct SessionManager {
    store: Arc<dyn SessionStore>,
}

#[post("/login")]
async fn login(
    body: Json<LoginRequest>,
    sessions: Inject<SessionManager>,
) -> Result<Json<SessionResponse>, HttpError> {
    // Authenticate user
    let user = authenticate(&body.email, &body.password).await?;

    // Create session
    let mut session = sessions.store.create().await.map_err(|e| {
        HttpError::internal(format!("Session creation failed: {e}"))
    })?;

    session.insert("user_id", user.id)?;
    session.insert("email", &user.email)?;
    session.insert("role", &user.role)?;
    sessions.store.save(&session).await.map_err(|e| {
        HttpError::internal(format!("Session save failed: {e}"))
    })?;

    Ok(Json(SessionResponse {
        session_id: session.id().to_string(),
    }))
}

#[post("/logout")]
async fn logout(
    session_id: Query<SessionIdParam>,
    sessions: Inject<SessionManager>,
) -> Result<StatusCode, HttpError> {
    sessions
        .store
        .delete(&session_id.id)
        .await
        .map_err(|e| HttpError::internal(format!("Session delete failed: {e}")))?;
    Ok(StatusCode::NO_CONTENT)
}
```

## Real-world pattern: flash messages

```rust
#[post("/submit")]
async fn submit_form(sessions: Inject<SessionManager>) -> Result<StatusCode, HttpError> {
    // ... process form ...

    // Store a flash message for the next request
    let mut session = sessions.store.create().await?;
    session.insert("flash_message", "Form submitted successfully")?;
    session.insert("flash_type", "success")?;
    sessions.store.save(&session).await?;

    Ok(StatusCode::FOUND)
}

#[get("/dashboard")]
async fn dashboard(
    session_id: Query<SessionIdParam>,
    sessions: Inject<SessionManager>,
) -> Result<Json<DashboardResponse>, HttpError> {
    let mut session = sessions.store.get(&session_id.id).await?;

    // Read and clear flash message
    let flash: Option<String> = session.remove("flash_message");
    let flash_type: Option<String> = session.remove("flash_type");
    sessions.store.save(&session).await?;

    Ok(Json(DashboardResponse { flash, flash_type }))
}
```

## SessionStore trait

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self) -> Result<Session, SessionError>;
    async fn get(&self, id: &SessionId) -> Result<Session, SessionError>;
    async fn save(&self, session: &Session) -> Result<(), SessionError>;
    async fn delete(&self, id: &SessionId) -> Result<(), SessionError>;
}
```

Implement this trait to create custom session backends (e.g., PostgreSQL, SQLite):

```rust
struct PgSessionStore {
    pool: sqlx::PgPool,
}

#[async_trait]
impl SessionStore for PgSessionStore {
    async fn create(&self) -> Result<Session, SessionError> {
        let session = Session::new();
        sqlx::query("INSERT INTO sessions (id, data, expires_at) VALUES ($1, '{}', now() + interval '24 hours')")
            .bind(session.id().to_string())
            .execute(&self.pool)
            .await?;
        Ok(session)
    }

    // ... get, save, delete ...
}
```

## Choosing a store

| Store | Persistence | Concurrency | Use case |
|-------|-------------|-------------|----------|
| `InMemorySessionStore` | None | Single process | Development, testing |
| `RedisSessionStore` | Redis (RDB/AOF) | Multiple replicas | Production HA |
| Custom (SQLx etc.) | Database | Multiple replicas | When Redis is unavailable |

## Session TTL best practices

| Use case | Recommended TTL | Reason |
|----------|----------------|--------|
| Banking/finance | 15 minutes | High security requirement |
| Standard web app | 24 hours | Good balance of UX and security |
| "Remember me" | 30 days | Long-lived, use refresh tokens |
| Shopping cart | 7 days | Users expect carts to persist |

The TTL is **renewed** every time `save()` is called, so active users don't
lose their session.

## Testing session logic

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_create_and_get() {
        let store = Arc::new(InMemorySessionStore::new());
        let session = store.create().await.unwrap();
        let loaded = store.get(session.id()).await.unwrap();
        assert_eq!(session.id(), loaded.id());
    }

    #[tokio::test]
    async fn test_session_insert_and_retrieve() {
        let store = Arc::new(InMemorySessionStore::new());
        let mut session = store.create().await.unwrap();
        session.insert("key", "value").unwrap();
        store.save(&session).await.unwrap();
        let loaded = store.get(session.id()).await.unwrap();
        assert_eq!(loaded.get::<String>("key").unwrap(), "value");
    }

    #[tokio::test]
    async fn test_session_delete() {
        let store = Arc::new(InMemorySessionStore::new());
        let session = store.create().await.unwrap();
        let id = session.id().clone();
        store.delete(&id).await.unwrap();
        let result = store.get(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_redis_store_roundtrip() {
        let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let conn = redis::aio::ConnectionManager::new(client).await.unwrap();
        let store = RedisSessionStore::new(conn);

        let mut session = store.create().await.unwrap();
        session.insert("user_id", 42u64).unwrap();
        store.save(&session).await.unwrap();

        let loaded = store.get(session.id()).await.unwrap();
        assert_eq!(loaded.get::<u64>("user_id").unwrap(), 42);
    }
}
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Forgetting `redis` feature with `RedisSessionStore` | Add `ironic = { features = ["redis"] }` to `Cargo.toml` |
| Redis not running at startup | Start Redis: `docker run -p 6379:6379 redis:7-alpine` |
| Session lost after restart in production | Use `RedisSessionStore` instead of `InMemorySessionStore` |
| TTL never set, sessions live forever | Always configure `session_ttl` in `RedisSessionConfig` |
| Session data too large for Redis | Redis recommends values under 500 KiB; store references instead |
| Session not saved after mutations | Call `store.save(&session)` after every `insert()` / `remove()` |
| Using `InMemorySessionStore` with multiple replicas | Sessions are per-process — switch to `RedisSessionStore` |

## What you learned

- [x] `InMemorySessionStore` for development — no persistence, single process
- [x] `RedisSessionStore` for production — Redis-backed, multi-replica safe
- [x] Session data is JSON-serialized under `ironic:session:{id}` keys
- [x] Configurable TTL (default 24h), renewed on each `save()`
- [x] Custom backends via `SessionStore` trait (PostgreSQL, SQLite, etc.)
- [x] Session TTL should match your security requirements
- [x] Always call `save()` after mutations, or data is lost
