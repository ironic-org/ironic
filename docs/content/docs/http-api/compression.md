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

## What you learned

- [x] `.compression()` enables gzip/brotli/zstd automatically
- [x] The best format is negotiated per client
- [x] Works with zero additional configuration
