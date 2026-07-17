---
title: Paginated Responses
description: Standardized pagination with items, total, offset, limit — built into FrameworkResponse.
---

# Paginated Responses

## What is it?

Every list endpoint needs pagination. Without it, returning all records at once overwhelms the client and the database. `FrameworkResponse::paginated()` gives you a standardized envelope.

## How to use

```rust
use ironic::prelude::*;

#[get("/users")]
async fn list(
    &self,
    #[query] page: Option<u64>,
    #[query] size: Option<u64>,
) -> Result<Json<Value>, HttpError> {
    let page = page.unwrap_or(1);
    let size = size.unwrap_or(20).min(100);
    let offset = (page - 1) * size;

    let items = self.service.list(offset, size)?;
    let total = self.service.count()?;

    FrameworkResponse::paginated(&items, total, offset, size)
}
```

**Response:**
```json
{
    "items": [{ "id": 1, "name": "Alice" }, { "id": 2, "name": "Bob" }],
    "total": 150,
    "offset": 0,
    "limit": 20
}
```

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `items` | `&[T]` | The current page of results |
| `total` | `u64` | Total number of items across all pages |
| `offset` | `u64` | Zero-based offset of this page |
| `limit` | `u64` | Page size |

## Try it

1. Create a route that returns a paginated list
2. Test `?page=1&size=10` — see 10 items, total > 10
3. Test `?page=99&size=10` — see empty items, same total
4. Verify `total` stays consistent across pages
