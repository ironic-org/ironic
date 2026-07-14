---
title: API Versioning
description: Version your API endpoints using URI prefixes, HTTP headers, or media types — without breaking existing clients.
---

# API Versioning

## What you'll learn

- Add version prefixes to your API routes (e.g., `/v2/items`)
- Use header-based or media-type versioning
- Run multiple versions of the same endpoint simultaneously
- Upgrade clients gradually without breaking anything

## The big picture

When you change your API, old clients need to keep working. Versioning lets you run **v1 and v2 side by side**:

```
Client A (old) ──► GET /v1/items ──► Returns old format
Client B (new) ──► GET /v2/items ──► Returns new format
```

## URI versioning (simplest)

Add a version prefix to your route URLs:

```rust
use ironic::prelude::*;
use ironic::{VersionMetadata, VersioningStrategy};

#[controller("/items")]
#[derive(Injectable)]
struct ItemsController { /* ... */ }

// v2 version of the same controller
ControllerDefinition::new::<ItemsController>("/items", provider)
    .unwrap()
    .version(VersionMetadata::new("2", VersioningStrategy::Uri))
    // Creates: /v2/items
```

Now both `/items` (v1) and `/v2/items` (v2) work:

```bash
curl http://localhost:3000/items         # → v1 response
curl http://localhost:3000/v2/items       # → v2 response
```

## Header versioning

Clients specify the version in a request header:

```rust
ControllerDefinition::new::<ItemsController>("/items", provider)
    .unwrap()
    .version(VersionMetadata::new("2", VersioningStrategy::Header))
    // Matches: Accept-Version: 2
```

```bash
curl -H "Accept-Version: 2" http://localhost:3000/items    # → v2
curl http://localhost:3000/items                            # → v1 (default)
```

## Media-type versioning

Clients specify the version in the Accept header:

```rust
ControllerDefinition::new::<ItemsController>("/items", provider)
    .unwrap()
    .version(VersionMetadata::new("2", VersioningStrategy::MediaType))
    // Matches: Accept: application/vnd.myapp.v2+json
```

## Multiple versions on the same controller

You can register the same controller multiple times with different versions:

```rust
let v1 = ControllerDefinition::new::<ItemsController>("/items", provider.clone())
    .unwrap()
    .version(VersionMetadata::new("1", VersioningStrategy::Uri));

let v2 = ControllerDefinition::new::<ItemsController>("/items", provider)
    .unwrap()
    .version(VersionMetadata::new("2", VersioningStrategy::Uri));
```

## Which strategy should I use?

| Strategy | When to use | Example |
|----------|------------|---------|
| **URI** | Public APIs, simple to understand | `/v1/items`, `/v2/items` |
| **Header** | Internal services, cleaner URLs | `Accept-Version: 2` |
| **Media-Type** | REST purists, content negotiation | `Accept: application/vnd.api.v2+json` |

> **Start with URI versioning.** It's the easiest for clients to understand and debug.

## Try it yourself

1. Create a controller at `/greet` that returns `"Hello v1"`
2. Add a v2 version that returns `"Hello v2"`
3. Test both: `curl /greet` and `curl /v2/greet`
4. Add a v3 that uses header versioning

## What you learned

- [x] Add URI prefixes with `VersioningStrategy::Uri`
- [x] Use header-based versioning with `VersioningStrategy::Header`
- [x] Run multiple API versions side by side
- [x] Choose the right versioning strategy for your use case
