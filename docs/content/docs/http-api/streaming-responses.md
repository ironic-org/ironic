---
title: Streaming Responses
description: Use shared body ownership for efficient large response cloning with Body::Stream.
---

# Streaming Responses

## What is it?

When building large responses (file downloads, report exports, bulk data), cloning the entire `Vec<u8>` body for every middleware/interceptor is wasteful. `Body::Stream(Arc<Vec<u8>>)` uses shared ownership — the body is cloned by incrementing a reference count, not by copying megabytes.

## How to use

```rust
use std::sync::Arc;
use ironic::prelude::*;

fn export_csv() -> Response {
    let csv_data: Vec<u8> = generate_large_csv_report();
    let shared_body = Arc::new(csv_data);
    Response::from_stream(HttpStatus::OK, shared_body)
}
```

## When to use

| Body size | Use |
|-----------|-----|
| < 10 KB | `Response::bytes()` — simple |
| 10 KB - 1 MB | `Response::json()` — standard JSON |
| > 1 MB | `Response::from_stream()` — shared ownership |

## How it works

- `Body::Bytes(Vec<u8>)` — owned body, clones copy all bytes
- `Body::Stream(Arc<Vec<u8>>)` — shared body, clones increment a reference count (atomic)

Middleware and interceptors that read the body can call `.as_bytes()` on either variant — they get a `&[u8]` slice regardless.

## Try it

1. Create an endpoint that returns 50,000 rows of generated data
2. Compare memory usage with `bytes()` vs `from_stream()`
3. Verify response body is correct with both approaches
