---
title: Pagination Extractor
description: Built-in `#[decorator(Pagination)]` extracts `?page=N&size=M` with proper URL decoding, clamping, and offset/limit helpers.
---

# Pagination Extractor

## What is it?

A `ParameterExtractor` that parses `?page=N&size=M` from the query string. Use it with `#[decorator(Pagination)]` on any handler parameter to get zero-boilerplate pagination.

## Quick Start

```rust
use ironic::prelude::*;

#[get("")]
async fn list(&self, #[decorator(Pagination)] p: Pagination) -> Response {
    let posts = self.service.list(p.offset(), p.limit()).await?;
    Response::json(200, &posts)
}
```

## API Reference

| Method | Description |
|--------|-------------|
| `Pagination::new()` | Creates with defaults (page=1, size=20, max=100) |
| `.max_size(max)` | Sets the maximum page size (clamped on extraction) |
| `.offset()` | Returns `(page - 1) * size` for SQL offset |
| `.limit()` | Returns `size` for SQL limit |

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `page` | `u64` | `1` | Current page (1-based, minimum 1) |
| `size` | `u64` | `20` | Items per page (minimum 1, clamped to `max_size` if set) |

## Custom Max Size

```rust
// Limit to 50 items per page
let extractor = Pagination::new().max_size(50);

#[get("")]
#[decorator(Pagination)]
async fn list_restricted(&self, p: Pagination) -> Response {
    // size is clamped to 50 even if ?size=500
    ...
}
```

The default max size is 100. Use `.max_size()` to override.

## Integration with Response::paginated()

Combine with `Response::paginated()` for a complete list endpoint:

```rust
#[get("")]
async fn list(&self, #[decorator(Pagination)] p: Pagination) -> Response {
    let items = self.service.list(p.offset(), p.limit()).await?;
    let total = self.service.count().await?;
    Response::paginated(&items, total, p.offset(), p.limit())
}
```
