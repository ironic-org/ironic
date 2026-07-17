---
title: Cache Invalidation
description: Remove cached entries by key prefix — invalidate user:*, product:*, or any pattern in one call.
---

# Cache Invalidation

## What is it?

`Cache::remove_by_prefix()` removes all cache entries whose keys start with a given prefix. Instead of tracking individual keys, invalidate entire groups at once.

## How to use

```rust
use ironic::services::cache::InMemoryCache;

let cache = InMemoryCache::new(1024);

cache.set("user:42", user_data, Some(Duration::from_secs(3600))).await?;
cache.set("user:43", user_data, Some(Duration::from_secs(3600))).await?;

let removed = cache.remove_by_prefix("user:").await?;
// removed = 2 — both user:* entries deleted
```

## Redis support

Works with `RedisCache` too:

```rust
let redis = RedisCache::new(conn).with_prefix("app");
redis.remove_by_prefix("user:").await?;
// Deletes app:user:* keys from Redis
```

## Common patterns

| Pattern | When |
|---------|------|
| `"user:"` | User schema changed — invalidate all user caches |
| `"product:category:electronics:"` | Category updated — invalidate that category's products |
| `"page:"` | Content refresh — invalidate all cached pages |

## What you learned

- [x] `remove_by_prefix()` deletes all matching keys in one async call
- [x] Works with both `InMemoryCache` and `RedisCache`
- [x] Returns the count of removed entries
