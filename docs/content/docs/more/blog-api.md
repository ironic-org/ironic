---
title: Blog API Example
description: A complete blog API built with Ironic — cross-module dependency injection, CRUD, categories, stats, search, and testing.
---

# Blog API Example

The blog-api example is a fully-functional blog platform that demonstrates real-world Ironic patterns: cross-module DI, in-memory repositories, filtering, categories, and a separate stats module consuming services from another module.

## What you'll learn

- Cross-module dependency injection — one module imports and uses another module's service
- Full CRUD with DTO validation using `garde`
- In-memory repositories with thread-safe state
- Service layer business logic (publish/unpublish, slug management, category assignment)
- Filtering, search, and tag breakdown
- Testing services without the DI container

## Project structure

```
examples/blog-api/
├── Cargo.toml
└── src/
    ├── main.rs               # Application entry point
    ├── app.rs                # Root module — imports all modules
    ├── welcome.rs            # Homepage endpoint
    ├── platform/config.rs    # Server configuration
    └── modules/
        ├── blogs/            # Primary module — blog posts + categories
        │   ├── entities/     # BlogPost, Category structs
        │   ├── dto/          # CreateBlogDto, UpdateBlogDto, BlogFilterDto
        │   ├── repositories/ # BlogRepository, CategoryRepository (in-memory)
        │   ├── services/     # BlogService — all business logic
        │   ├── controller/   # BlogsController, CategoriesController
        │   └── tests/        # 9 unit tests
        └── stats/            # Cross-module DI demo — consumes BlogService
            ├── services/     # StatsService (injects Arc<BlogService>)
            └── controller/   # StatsController
```

## Cross-module dependency injection

The `StatsModule` imports `BlogsModule` and injects `BlogService` into `StatsService`:

```rust
// modules/stats/mod.rs
#[derive(Module)]
#[module(
    imports = [crate::modules::blogs::BlogsModule],  // ← imports the module
    providers = [StatsService],
    controllers = [StatsController],
)]
pub struct StatsModule;
```

```rust
// modules/stats/services/stats_service.rs
#[derive(Injectable)]
pub struct StatsService {
    blog_service: Arc<BlogService>,  // ← resolved from BlogsModule's exports
}
```

`StatsService` calls `BlogService` directly to aggregate data:

```rust
impl StatsService {
    pub fn tag_breakdown(&self) -> Result<serde_json::Value, HttpError> {
        let posts = self.blog_service.list(&BlogFilterDto::default())?;
        // count tags across all posts...
    }
}
```

The key chain:
1. `BlogsModule` declares `exports = [BlogService]`
2. `StatsModule` declares `imports = [BlogsModule]`
3. `StatsService` declares `blog_service: Arc<BlogService>` — the container resolves it automatically

## Running the example

```bash
git clone https://github.com/ironic-org/ironic
cd ironic/examples/blog-api
SERVER_PORT=3002 cargo run
```

The API starts at `http://localhost:3002`.

## API reference

### Blog posts

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/blogs` | List posts (with filters) |
| `POST` | `/api/blogs` | Create a post |
| `GET` | `/api/blogs/:id` | Get a post by UUID |
| `PUT` | `/api/blogs/:id` | Update a post |
| `DELETE` | `/api/blogs/:id` | Delete a post |
| `GET` | `/api/blogs/slug/:slug` | Get a post by slug |
| `POST` | `/api/blogs/:id/publish` | Publish a post |
| `POST` | `/api/blogs/:id/unpublish` | Unpublish a post |
| `GET` | `/api/blogs/stats` | Blog statistics |
| `GET` | `/api/blogs/:id/categories` | Get a post's categories |
| `POST` | `/api/blogs/:id/categories/:category_id` | Add category to post |
| `DELETE` | `/api/blogs/:id/categories/:category_id` | Remove category from post |

### Filtering

`GET /api/blogs` accepts query parameters:

```json
{
  "published": true,
  "author": "Alice",
  "tag": "rust",
  "category_id": "uuid",
  "search": "keyword"
}
```

### Categories

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/categories` | List all categories |
| `POST` | `/api/categories` | Create a category |
| `DELETE` | `/api/categories/:id` | Delete a category |

### Cross-module stats

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/stats/blog` | Blog summary (total, published, drafts, words, tags) |
| `GET` | `/api/stats/blog/tags` | Tag frequency breakdown |

## Key patterns

### In-memory repository

Repositories use `LazyLock<Mutex<HashMap>>` for thread-safe in-memory storage. This keeps the example self-contained without a database:

```rust
static BLOG_POSTS: LazyLock<Mutex<HashMap<Uuid, BlogPost>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
```

### Slug generation

Titles are converted to URL-friendly slugs with duplicate detection:

```rust
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}
```

### DTO validation

Create requests are validated at the boundary:

```rust
#[derive(Validate)]
pub struct CreateBlogDto {
    #[garde(length(min = 1, max = 200))]
    pub title: String,

    #[garde(length(min = 1))]
    pub content: String,
}
```

## Testing

The module includes 9 unit tests covering CRUD, slug conflicts, publish/unpublish workflow, category assignment, and stats. Tests construct `BlogService` directly without the DI container:

```rust
fn make_service() -> BlogService {
    BlogService {
        blog_repo: Arc::new(BlogRepository),
        category_repo: Arc::new(CategoryRepository),
    }
}
```

Run them:

```bash
cargo test --package blog-api
```

## What you learned

- [x] Cross-module DI — `StatsModule` imports and uses `BlogService` from `BlogsModule`
- [x] Full CRUD with `Injectable` services and repositories
- [x] In-memory state with thread-safe `Mutex<HashMap>`
- [x] DTO validation with `garde`
- [x] Slug generation and duplicate detection
- [x] Sub-resource routing (categories nested under posts)
- [x] Service-layer testing without the DI container
