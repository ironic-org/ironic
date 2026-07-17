---
title: Caching
description: Cache route responses with `#[cache(ttl_secs = ...)]` and the `CacheInterceptor`.
---

# Caching

## What you'll learn

- Cache route responses with the `#[cache]` attribute
- Use the `CacheInterceptor` with in-memory or Redis backends
- Fine-tune TTL and cache key behavior

---

## Enabling

```toml
ironic = { features = ["cache"] }
```

For Redis-backed caching, add `"redis"`:

```toml
ironic = { features = ["cache", "redis"] }
```

---

## Route-level cache attribute

Use `#[cache(ttl_secs = N)]` on a handler method to annotate it with cache metadata:

```rust
use ironic::prelude::*;

#[controller("/products")]
#[derive(Injectable)]
struct ProductController {
    service: Arc<ProductService>,
}

#[routes]
impl ProductController {
    #[get("/")]
    #[cache(ttl_secs = 60)]
    async fn list(&self) -> Result<Json<Vec<Product>>, HttpError> {
        self.service.list_all().await
    }

    #[get("/{id}")]
    #[cache(ttl_secs = 300)]
    async fn show(&self, #[param] id: u64) -> Result<Json<Product>, HttpError> {
        self.service.find(id).await
    }
}
```

The attribute stores `CacheMetadata { ttl_secs }` on the route, which the `CacheInterceptor` reads at runtime.

---

## CacheInterceptor

Register the interceptor globally to enable caching:

```rust
use ironic::services::cache::{CacheInterceptor, InMemoryCache};
use std::sync::Arc;

let app = CompiledHttpApplication::new(container, routes)
    .interceptor(CacheInterceptor::new(Arc::new(InMemoryCache::new(1024))));
```

### InMemoryCache

Process-local cache with a fixed-capacity LRU:

```rust
use ironic::services::cache::InMemoryCache;

// Default capacity: 1024 entries
let cache = InMemoryCache::default();

// Custom capacity
let cache = InMemoryCache::new(4096);

// JSON typed helpers
let user: Option<User> = cache.get_json("user:42").await?;
cache.set_json("user:42", &user, Some(Duration::from_secs(300))).await?;
```

### RedisCache

Persistent cache backed by Redis:

```rust
use ironic::services::cache::RedisCache;

let client = redis::Client::open("redis://localhost")?;
let conn_mgr = client.get_connection_manager().await?;
let cache = RedisCache::new(conn_mgr).with_prefix("myapp");

let app = CompiledHttpApplication::new(container, routes)
    .interceptor(CacheInterceptor::new(Arc::new(cache)));
```

---

## How it works

1. `#[cache(ttl_secs = 60)]` stores `CacheMetadata` on the `RouteDefinition`
2. `CacheInterceptor::intercept()` reads the metadata from `route_metadata()`
3. On first request: handler runs, response body is stored with the TTL
4. Subsequent requests: cached response is returned immediately — handler is never called
5. After TTL expires: next request re-runs the handler and refreshes the cache

**Cache key format:** `route-cache:{ttl_secs}:{METHOD}:{PATH}`

---

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Forgetting to register `CacheInterceptor` | Add `.interceptor(CacheInterceptor::new(...))` to the application |
| Cache not invalidating | Each unique TTL+method+path is a separate key — no manual invalidation is exposed |
| Using `#[cache]` without `cache` feature | Enable `cache` (or `application-services`) in `Cargo.toml` |
| Passing `Arc<dyn Cache>` directly to `CacheInterceptor::new()` | Accepts any `impl Cache` — wrap with `Arc::new(InMemoryCache::new(...))` |

## What you learned

- [x] `#[cache(ttl_secs = N)]` annotates routes with cache metadata
- [x] `CacheInterceptor` reads the metadata and serves cached responses
- [x] `InMemoryCache` for process-local caching, `RedisCache` for persistent caching
- [x] Cache key is `route-cache:{ttl}:{METHOD}:{PATH}`
- [x] Cache is automatically invalidated after TTL expires
