---
title: Compression
description: Compress API responses with gzip, brotli, or zstd — reduce bandwidth and speed up clients.
---

# Compression

## What is it?

When a client (browser, mobile app, curl) makes an HTTP request, it tells the server which compression formats it understands via the `Accept-Encoding` header:

```
GET /api/blogs HTTP/1.1
Accept-Encoding: gzip, brotli
```

The server can respond with a compressed body and a `Content-Encoding` header:

```
HTTP/1.1 200 OK
Content-Encoding: brotli
[brotli-compressed bytes...]
```

This is **transparent** — the client decompresses automatically. Your handler code doesn't know or care that compression happened. The result: JSON payloads shrink 5-10x, saving bandwidth and making your API feel faster.

## Enabling

```toml
ironic = { features = ["compression"] }
```

Add one line to enable:

```rust
FrameworkApplication::builder()
    .platform(AxumAdapter::new().compression())
    .build().await.unwrap();
```

That's it. No levels, no thresholds, no per-route configuration needed. The framework automatically negotiates the best format:

| Client sends `Accept-Encoding` | Server responds with |
|---|---|
| `br, gzip` | brotli (best compression) |
| `gzip` only | gzip |
| `zstd, gzip` | zstd |
| nothing | uncompressed |

## How it works under the hood

- `.compression()` sets a flag on `AxumAdapter`
- During `build()`, if enabled, a `tower_http::CompressionLayer` is inserted into the middleware stack
- The layer reads `Accept-Encoding`, applies the best supported algorithm, and adds `Content-Encoding` to the response
- Compression runs **after** your handler — your code always works with plain bytes

## Layer order

In the request pipeline, compression sits near the outer edge:

```
SecurityHeaders → RateLimit → CORS → Metrics → Compression → BodyLimit → Timeout → Router
```

This means compression wraps the entire response after all other layers have processed it.

## When to use

Always enable compression in production. The CPU cost is negligible compared to network savings. JSON API responses typically compress 5-10x smaller.

## When NOT to use

| Scenario | Reason |
|---|---|
| Already-compressed content (PNG, JPEG, MP4, ZIP) | Re-compressing adds CPU with no size reduction |
| Tiny responses (< 100 bytes) | Gzip headers alone can be 20+ bytes — output may be *larger* |
| WebSocket connections | Per-message compression is a different protocol |
| Streaming / SSE | Chunked responses must flush immediately; compression buffers |

## Common mistakes

| Mistake | Fix |
|---|---|
| Not enabling compression in production | Add `.compression()` to your `AxumAdapter` |
| Expecting fine-grained control | The current API is a simple on/off toggle — future versions may add level/threshold control |

## What you learned

- [x] `.compression()` enables gzip/brotli/zstd with one call
- [x] The best format is auto-negotiated per client
- [x] Works with zero additional configuration
- [x] Enable it in production for 5-10x smaller responses
