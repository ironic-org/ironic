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

## Backends

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

## Try it yourself

1. Add `#[cache(ttl_secs = 30)]` to a route
2. Call the route twice in 10 seconds — second call should be instant
3. Wait 30 seconds and call again — should recompute

## What you learned

- [x] `#[cache(ttl_secs = N)]` caches route responses
- [x] `InMemoryCache` for development, `RedisCache` for production
- [x] Cache is automatic — no manual get/set needed
