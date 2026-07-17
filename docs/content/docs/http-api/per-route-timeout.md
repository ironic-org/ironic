---
title: Per-Route Timeout
description: Override the global request timeout per route — long uploads, quick health checks, and everything in between.
---

# Per-Route Timeout

## What is it?

The global `AxumAdapter::request_timeout()` applies to every route. But different routes need different timeouts. `.timeout()` on `RouteDefinition` overrides it per route.

## How to use

```rust
RouteDefinition::new(HttpMethod::GET, "/health", "health", handler)?
    .timeout(Duration::from_secs(1)); // fast

RouteDefinition::new(HttpMethod::POST, "/upload", "upload", handler)?
    .timeout(Duration::from_secs(300)); // 5 min for uploads
```

## Priority

```
Route-level timeout > Global adapter timeout
```

If no per-route timeout is set, the global default applies.

## Best practices

| Route type | Recommended timeout |
|-----------|-------------------|
| Health check | 1-2 seconds |
| Standard CRUD | Global default (30s) |
| File upload | 60-300 seconds |
| Report export | 120-600 seconds |
| WebSocket upgrade | No timeout |
