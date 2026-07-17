---
title: Caching
description: Speed up your API with automatic caching — cache route responses with a single attribute.
---

# Caching

## What you'll learn

- Cache route responses with `#[cache]` attribute
- Choose between in-memory and Redis cache backends
- Set TTL (time-to-live) per route
- Invalidate cache when data changes
- Control cache keys and response headers

Enable in `Cargo.toml`:

```toml
ironic = { features = ["cache"] }
```

---

## Quick start

```rust
#[controller("/products")]
#[derive(Injectable)]
struct ProductsController {
    service: Arc<ProductsService>,
}

#[routes]
impl ProductsController {
    #[get("/:id")]
    #[cache(ttl_secs = 60)]              // ← Cache this response for 60 seconds
    async fn get(&self, #[param] id: u64) -> Result<Json<Product>, HttpError> {
        self.service.find(id).map(Json)
    }
}
```

That's it! The framework:
1. Checks cache before calling `find()` — if cached, returns instantly
2. After calling `find()`, stores the result in cache for 60 seconds
3. After 60 seconds, the next request recomputes and re-caches

## Cache key strategy

By default the framework generates cache keys from the route path and request method:

```
GET:/products/42 → cache_key = "GET:/products/42"
```

Query parameters are **not** included by default, which means `/products?page=1` and `/products?page=2` share the same cache entry — usually a bug. Include query params explicitly:

```rust
#[get("/products")]
#[cache(ttl_secs = 30, cache_key = "{path}?{query}")]
async fn list(&self, #[query] page: Option<u32>) -> Result<Json<Vec<Product>>, HttpError> {
    self.service.list(page).map(Json)
}
```

The `{path}` and `{query}` placeholders expand at runtime. You can also build fully custom keys:

```rust
#[cache(ttl_secs = 30, cache_key = "products:page:{page}")]
async fn list(&self, #[query] page: Option<u32>) -> Result<Json<Vec<Product>>, HttpError> {
    // ...
}
```

Route parameters, query params, and header values are all available for key interpolation.

## TTL configuration and best practices

TTL (time-to-live) is the most important caching decision. Match it to your data's change frequency:

| Data type | Recommended TTL | Why |
|-----------|----------------|-----|
| Static content (categories, config) | 300-3600s (5-60 min) | Rarely changes |
| Semi-dynamic (product details, blog posts) | 30-300s (30s-5 min) | Changes a few times per day |
| Dynamic (search results, dashboards) | 5-30s | Acceptable staleness window |
| Real-time (stock prices, auth state) | Do NOT cache | Every request must be fresh |

The `ttl_secs` parameter accepts any positive integer. Zero or negative values disable the cache for that route (though using `#[hide_cache]` is clearer):

```rust
#[get("/dashboard")]
#[cache(ttl_secs = 15)]   // 15-second staleness for dashboards is usually fine
async fn dashboard(&self) -> Result<Json<DashboardData>, HttpError> {
    self.analytics.aggregate().map(Json)
}
```

## Cache invalidation

There are two invalidation strategies:

### TTL-based expiry (automatic)

All `#[cache(ttl_secs = N)]` entries expire automatically after N seconds. This is the simplest strategy — set a reasonable TTL and let entries age out.

### Manual invalidation

Purge specific keys when you know data changed:

```rust
use ironic::cache::Cache;

#[post("/products")]
async fn create(&self, body: Json<CreateProduct>) -> Result<Json<Product>, HttpError> {
    let product = self.service.create(body.0).await?;

    // Invalidate the product list cache so the next GET returns fresh data
    self.cache.delete("GET:/products").await;

    Ok(Json(product))
}

#[put("/products/:id")]
async fn update(&self, #[param] id: u64, body: Json<UpdateProduct>) -> Result<Json<Product>, HttpError> {
    let product = self.service.update(id, body.0).await?;

    // Invalidate both the list and the individual item
    self.cache.delete("GET:/products").await;
    self.cache.delete(&format!("GET:/products/{}", id)).await;

    Ok(Json(product))
}
```

The `Cache` trait gives you `get`, `set`, `delete`, and `clear` methods. Inject it via the DI container like any other service.

## Multi-backend setup

| Backend | Feature | Best for |
|---------|---------|----------|
| `InMemoryCache` | `cache` (default) | Single server, development |
| `RedisCache` | `redis` + `cache` | Multi-server, production |

```rust
// Development (in-memory):
ironic = { features = ["cache"] }

// Production (Redis):
ironic = { features = ["cache", "redis"] }
```

### Choosing the right backend

| Scenario | Use |
|----------|-----|
| Single instance, low traffic | In-memory — zero configuration, no network overhead |
| Multiple instances behind a load balancer | Redis — all instances share one cache, so a cache fill from server A is visible to server B |
| Serverless / ephemeral containers | Redis — in-memory caches vanish when containers restart |
| Development and testing | In-memory — no Redis dependency to set up |

Configure Redis in your application builder:

```rust
Application::builder()
    .cache(RedisCache::new("redis://localhost:6379").await?)
    .build().await.unwrap();
```

## Cache-Control headers

Use the `#[cache_control]` attribute to set HTTP response headers that tell clients and proxies how to cache:

```rust
#[get("/products")]
#[cache(ttl_secs = 120)]
#[cache_control(public, max_age = 120)]    // CDN and browser may also cache
async fn list(&self) -> Result<Json<Vec<Product>>, HttpError> {
    self.service.list().map(Json)
}
```

| Directive | Effect |
|-----------|--------|
| `public` | Allow CDNs and proxies to cache the response |
| `private` | Only the browser may cache it |
| `max_age = N` | How long (seconds) before the client must revalidate |
| `no_cache` | Client must revalidate before using the cached copy |
| `no_store` | Do not cache at all |

The server-side `#[cache]` TTL and the `Cache-Control` `max_age` should usually match. A mismatch means your server cache is fresher than what the client sees, or vice versa.

## What NOT to cache

Caching the wrong data creates bugs that are hard to reproduce:

| Don't cache | Why |
|-------------|-----|
| User-specific data (`/me`, `/profile`) | User A would see User B's data. Always pass user ID in the cache key if you must cache these. |
| Real-time data (stock prices, bids, live feeds) | Staleness is unacceptable by definition |
| POST / PUT / DELETE responses | These modify state; caching the 201/200 response is meaningless |
| Error responses (4xx, 5xx) | Caching a 503 means the service appears down for the entire TTL window |
| Responses with `Authorization` header | Different users would share the same cached response |

## Common mistakes

| Mistake | Why it's wrong | Fix |
|---------|---------------|-----|
| Caching without query params in the key | `/products?page=1` returns same data as `?page=2` | Include `{query}` in your cache key |
| Using in-memory cache across multiple instances | Cache hits are only valid on the instance that stored them | Switch to Redis when running >1 instance |
| Not invalidating after mutations | Stale data is worse than no cache | Call `cache.delete()` in POST/PUT/DELETE handlers |
| Setting TTL too high | Users see outdated data for too long | Match TTL to data freshness requirements |
| Caching user-scoped endpoints | User A sees User B's cached data | Include user ID in the cache key |

## Try it yourself

1. Add `#[cache(ttl_secs = 30)]` to a route
2. Call the route twice in 10 seconds — second call should be instant
3. Wait 30 seconds and call again — should recompute
4. Add a mutation endpoint that calls `cache.delete()` and verify stale data is gone

## What you learned

- [x] `#[cache(ttl_secs = N)]` caches route responses
- [x] `InMemoryCache` for development, `RedisCache` for production
- [x] Cache is automatic — no manual get/set needed
- [x] Cache keys are built from path, method, and optionally query params
- [x] Manual invalidation via `cache.delete()` keeps data fresh after mutations
- [x] Redis is required for multi-instance deployments
- [x] `#[cache_control]` sends proper HTTP caching headers to clients
- [x] Never cache user-specific data, real-time data, or error responses
