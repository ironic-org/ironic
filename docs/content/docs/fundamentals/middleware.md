---
title: Middleware
description: Cross-cutting concerns in the request pipeline — CORS, compression, rate limiting, and custom middleware.
---

# Middleware

Middleware operates at the tower layer level, running before routing occurs. This makes them ideal for cross-cutting concerns that should apply to every request regardless of route.

## Middleware vs guards vs interceptors

| Concept | Position | Purpose |
|---------|----------|---------|
| **Middleware** | Before router (tower layers) | Cross-cutting: CORS, compression, rate limiting |
| **Guards** | After routing, before handler | Authorization |
| **Interceptors** | Around handler | Request/response transformation |
| **Pipes** | Inside handler params | Parameter validation |

## Built-in middleware

Ironic bundles several tower-http middleware layers:

### CORS

```rust
use ironic::security::cors::CorsConfig;

let config = CorsConfig {
    allowed_origins: vec!["https://example.com".into()],
    allowed_methods: vec!["GET".into(), "POST".into()],
    allowed_headers: vec!["Authorization".into()],
    max_age: Some(3600),
};

AxumAdapter::new()
    .configure_router(|router| {
        router.layer(CorsLayer::new(config))
    })
```

### Compression

```rust
use tower_http::compression::CompressionLayer;

AxumAdapter::new()
    .configure_router(|router| {
        router.layer(CompressionLayer::new())
    })
```

### Rate limiting

```rust
use ironic::security::rate_limit::InMemoryRateLimiter;

let limiter = InMemoryRateLimiter::new(100, Duration::from_secs(60));

AxumAdapter::new()
    .configure_router(|router| {
        router.layer(RateLimitMiddleware::new(limiter))
    })
```

### Security headers

```rust
use ironic::security::SecurityHeadersConfig;

AxumAdapter::new()
    .configure_router(|router| {
        router.layer(SecurityHeadersMiddleware::new(
            SecurityHeadersConfig::default()
        ))
    })
```

## Custom middleware

Any tower layer can be used with Ironic:

```rust
use tower::layer::Layer;

struct RequestTimerLayer;

impl<S> Layer<S> for RequestTimerLayer {
    type Service = RequestTimerService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RequestTimerService { inner: service }
    }
}

AxumAdapter::new()
    .configure_router(|router| {
        router.layer(RequestTimerLayer)
    })
```

## Middleware configuration

Middleware is configured at the adapter level:

```rust
AxumAdapter::new()
    .configure_router(|router| {
        router
            .layer(CompressionLayer::new())
            .layer(CorsLayer::new(cors_config))
            .layer(MetricsLayer::new(MetricsConfig::default()))
            .layer(SecurityHeadersMiddleware::new(security_config))
    })
    .build(app.compile())
```

## Ordering matters

Middleware runs in the order they're applied. The first layer added is the outermost layer:

```
Client → CompressionLayer → CorsLayer → MetricsLayer → Router → Handler
```

Choose the order carefully:
- **Compression** should be outermost (compress before sending)
- **CORS** should be early (reject cross-origin before processing)
- **Metrics** should be early (capture all requests including rejected ones)
- **Rate limiting** should be early (reject before processing)
