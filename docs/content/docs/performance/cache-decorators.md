---
title: Cache decorators
description: Cache handler responses with the cache interceptor and pluggable backends.
---

# Cache decorators

Enable `cache` to mark routes for response caching. The `CacheInterceptor` checks the cache
before the handler runs and populates the cache with the handler's response on a cache miss.

```toml
ironic = { features = ["cache"] }
```

## Enabling cache on a route

Attach `#[cache(ttl_secs = N)]` to a route handler in a `#[routes]` impl block:

```rust
#[routes]
impl ProductsController {
    #[cache(ttl_secs = 60)]
    #[get("/products")]
    async fn list(&self) -> Result<impl IntoFrameworkResponse, HttpError> {
        // Expensive database query — cached for 60 seconds
    }
}
```

The macro attaches `CacheMetadata` to the route definition. The `CacheInterceptor` reads this
metadata and applies caching transparently.

## CacheInterceptor

Register the interceptor on a controller or globally:

```rust
use ironic::{CacheInterceptor, services::cache::InMemoryCache};
use std::sync::Arc;

let interceptor = CacheInterceptor::new(Arc::new(InMemoryCache::new(1024)));
// ControllerDefinition::new(...).interceptor(interceptor)
```

The interceptor skips routes without `CacheMetadata`. On a cache hit, it returns the cached
response immediately. On a miss, it invokes the handler and stores the response body with the
configured TTL.

## Cache backends

### In-memory cache

```rust
use ironic::services::cache::InMemoryCache;

let cache = InMemoryCache::new(1024);
cache.set_json("key", &value, Some(Duration::from_secs(60))).await?;
let value = cache.get_json::<MyType>("key").await?;
```

The in-memory cache evicts expired entries and enforces a maximum capacity. It is suitable for
single-process deployments.

### Redis cache

Enable `redis` to use Redis as a distributed cache backend:

```toml
ironic = { features = ["cache", "redis"] }
```

```rust
use ironic::services::cache::RedisCache;
use redis::aio::ConnectionManager;

let client = ConnectionManager::new(client).await?;
let cache = RedisCache::new(client).with_prefix("myapp");
// Use with CacheInterceptor or directly via the Cache trait
```

## The `Cache` trait

Implement `Cache` for any backend:

```rust
use ironic::services::cache::{Cache, CacheFuture, CacheError};

impl Cache for MyCustomCache {
    fn get<'a>(&'a self, key: &'a str) -> CacheFuture<'a, Option<Vec<u8>>> { ... }
    fn set<'a>(&'a self, key: &'a str, value: Vec<u8>, ttl: Option<Duration>) -> CacheFuture<'a, ()> { ... }
    fn remove<'a>(&'a self, key: &'a str) -> CacheFuture<'a, bool> { ... }
    fn clear(&self) -> CacheFuture<'_, ()> { ... }
}
```
