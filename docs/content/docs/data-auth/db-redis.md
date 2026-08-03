---
title: Redis
description: Caching, rate limiting, and session-like patterns with Redis — plus manual ConnectionManager setup.
---

# Redis


## Feature flag

```toml
ironic = { features = ["redis"] }
```

## Connection URL

```
redis://user:password@localhost:6379
```

## Service

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

## Redis as a rate limiter

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

## Manual connection setup


When you need full control over pool configuration, build the connection manually and register it as a provider:

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
