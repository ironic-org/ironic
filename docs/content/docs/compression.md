---
title: Response compression
description: Serve compressed responses with gzip, brotli, or zstd via the Axum adapter.
---

# Response compression

Enable `compression` to compress responses with gzip, brotli, or zstd. Compression is applied as
a Tower layer through the Axum adapter and respects the client's `Accept-Encoding` header.

```toml
ironic = { features = ["compression"] }
```

## Enabling compression

```rust
use ironic::AxumAdapter;

let adapter = AxumAdapter::new().compression();
```

The layer automatically selects the best encoding from what the client advertises. Responses
smaller than approximately 1 KB and responses with non-compressible content types are skipped.

## Supported encodings

| Encoding | `Accept-Encoding` value | Priority |
|----------|------------------------|----------|
| Brotli   | `br`                   | Highest  |
| Gzip     | `gzip`                 | Medium   |
| Zstd     | `zstd`                 | Low      |

## Content-type filtering

Only responses with compressible media types are processed. Text formats (`text/html`,
`application/json`, `application/xml`, etc.) are compressed; binary formats (`image/*`,
`application/octet-stream`) pass through unchanged.

## Conditional compression

Exclude specific routes from compression by attaching metadata:

```rust
RouteDefinition::new(HttpMethod::GET, "/stream", "stream", handler_fn(handler))?
    .middleware(NoCompressionMiddleware);
```
