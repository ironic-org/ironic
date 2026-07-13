---
title: Security and production defaults
description: Request limits, CORS, security headers, rate limits, and secret handling.
---

# Security and production defaults

The Axum adapter buffers at most 1 MiB per request and applies a 30-second end-to-end timeout.
Override these only after measuring the endpoint:

```rust
use std::time::Duration;
use ironic::AxumAdapter;

let adapter = AxumAdapter::new()
    .request_body_limit(2 * 1024 * 1024)
    .request_timeout(Duration::from_secs(10));
```

## CORS

Use `AxumAdapter::configure_router` with `tower_http::cors::CorsLayer`. Do not use a wildcard origin
with credentials; enumerate trusted origins and methods.

## Security headers

Apply `tower_http::set_header` or a dedicated Tower layer for HSTS, content-type options, frame
policy, referrer policy, and a deployment-specific content security policy. Terminate TLS at a
trusted proxy or in the application before enabling HSTS.

## Rate limiting

Apply a Tower rate-limit layer at the adapter boundary. Key limits by authenticated principal when
available, use bounded state, and return `429 Too Many Requests` with retry guidance.

## Secrets

Load credentials from environment variables or a secret manager, wrap typed values in `Secret<T>`,
and never include exposed values in logs, errors, tracing fields, panic messages, or generated
configuration files. Rotate credentials independently of application releases.
