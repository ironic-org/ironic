---
title: Compression
description: Compress API responses with gzip, brotli, or zstd — reduce bandwidth and speed up clients.
---

# Compression

Enable in `Cargo.toml`:

```toml
ironic = { features = ["compression"] }
```

Add one line to enable:

```rust
FrameworkApplication::builder()
    .platform(AxumAdapter::new().compression())  // ← That's it!
    .build().await.unwrap();
```

The server automatically negotiates the best format the client supports:

| Client supports | Server sends |
|----------------|-------------|
| brotli + gzip | brotli (best compression) |
| gzip only | gzip |
| zstd + gzip | zstd |
| nothing | uncompressed |

> **When to use:** Always enable compression in production. Typical JSON API responses compress 5-10x smaller. The CPU cost is negligible compared to network savings.

## Per-route compression opt-out

Some endpoints should never be compressed — file downloads, streaming responses, or already-compressed content. Disable compression on specific routes:

```rust
#[get("/download/:filename")]
#[no_compression]                           // ← Skip compression for this route
async fn download(&self, #[param] filename: String) -> Result<Vec<u8>, HttpError> {
    self.storage.read(filename).await
}
```

This overrides the global `.compression()` only for the annotated route. Requests to `/download/*` are served uncompressed regardless of `Accept-Encoding`.

## Compression level control

Set the compression level directly on the adapter builder. Higher levels trade CPU for smaller output:

```rust
FrameworkApplication::builder()
    .platform(AxumAdapter::new()
        .compression()
        .compression_level(CompressionLevel::Balanced)  // Default
    )
    .build().await.unwrap();
```

| Level | Description |
|-------|-------------|
| `CompressionLevel::Fast` | Lowest CPU, larger output. Good for high-throughput APIs. |
| `CompressionLevel::Balanced` | Default. Good compromise for most workloads. |
| `CompressionLevel::Best` | Smallest output, highest CPU. Best for slow networks or edge delivery. |

When using both brotli and gzip, you can configure levels independently:

```rust
.compression_level_brotli(CompressionLevel::Best)
.compression_level_gzip(CompressionLevel::Fast)
```

This sends best-compression brotli to modern browsers and fast gzip to legacy clients, without wasting CPU on both.

## Size threshold

Compressing tiny responses wastes CPU for negligible gain. Set a minimum size:

```rust
AxumAdapter::new()
    .compression()
    .compression_min_size(1024)  // Only compress responses >= 1KB
```

Responses below the threshold (common for 204 No Content, empty arrays, or health checks) are sent uncompressed. A threshold of 512–1024 bytes is a good starting point.

## Compression algorithm comparison

| Algorithm | Compression ratio | Speed | Browser support | Best for |
|-----------|------------------|-------|----------------|----------|
| gzip | 3-5x | Fastest | Universal | Legacy clients, edge proxies |
| brotli | 5-7x | Moderate | 97% of browsers | Modern web apps, static assets |
| zstd | 4-8x | Fast | Growing (HTTP/3) | Internal microservices, streams |

Brotli typically produces 15-25% smaller output than gzip at a modest CPU cost. Zstd is the newest contender — competitive compression with gzip-like speed, ideal for server-to-server communication.

## When NOT to compress

Compression is not free. Skip it when:

| Scenario | Reason |
|----------|--------|
| Already-compressed content | PNG, JPEG, MP4, ZIP, WebP are already compressed — re-compressing adds CPU with no size reduction |
| Tiny responses (<512 bytes) | Gzip headers alone can be 20+ bytes; on small payloads, the "compressed" output may be *larger* |
| WebSocket connections | Per-message compression is available but different from HTTP response compression — use `.per_message_deflate()` instead |
| Streaming / SSE | Chunked responses must be flushed immediately; compression requires buffering |

## Common mistakes

| Mistake | Why it's wrong | Fix |
|---------|---------------|-----|
| Not enabling compression in production | JSON responses are 5-10x larger than needed | Add `.compression()` to your AxumAdapter |
| Compressing file downloads | Binary files are already compressed or benefit from range requests | Use `#[no_compression]` on file-serving routes |
| Using `Best` level everywhere | High CPU per request adds latency under load | Use `Balanced` for dynamic APIs, `Best` only for static assets with CDN caching |
| Setting `min_size` too high | Responses just above the threshold bloat your bandwidth | Keep it at 512–1024 bytes |

## What you learned

- [x] `.compression()` enables gzip/brotli/zstd automatically
- [x] The best format is negotiated per client
- [x] Works with zero additional configuration
- [x] `#[no_compression]` disables compression per route
- [x] `CompressionLevel` controls CPU vs ratio trade-off
- [x] `compression_min_size` avoids wasteful tiny-response compression
- [x] Don't compress images, videos, or other already-compressed content
