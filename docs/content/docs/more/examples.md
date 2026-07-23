---
title: Examples
description: Real-world example application built with Ironic — cross-module DI, CRUD with categories, in-memory repositories, stats module, filtering, and slug management.
---

# Examples

Each example is a complete, runnable project:

| Example | What it demonstrates |
|---------|---------------------|
| [blog](https://github.com/ironic-org/ironic/tree/main/examples/blog) | Cross-module DI, CRUD with categories, in-memory repositories, stats module, filtering, and slug management |

## Running the example

```bash
git clone https://github.com/ironic-org/ironic
cd ironic/examples/blog-api
SERVER_PORT=3002 cargo run
```

## blog-api

A complete blog platform demonstrating **cross-module dependency injection**:

- `BlogsModule` exports `BlogService`, `StatsModule` imports and uses it
- Blog post CRUD with title, content, excerpt, tags, and author
- Category management (create, list, delete, assign to posts)
- Slug generation with duplicate detection
- Publish/unpublish workflow
- Filtering by status, author, tag, category, and full-text search
- Tag frequency breakdown from a separate `StatsService`
- 9 unit tests covering all business logic

```rust
// StatsModule imports BlogsModule — cross-module DI
#[derive(Module)]
#[module(
    imports = [crate::modules::blogs::BlogsModule],
    providers = [StatsService],
    controllers = [StatsController],
)]
pub struct StatsModule;
```

## What you learned

- [x] `blog-api` = cross-module DI, sub-resource routing, and real-world patterns
