---
title: Examples
description: Real-world example applications built with Ironic — REST APIs, WebSockets, validation, error handling, cross-module DI, and testing.
---

# Examples

Each example is a complete, runnable project:

| Example | What it demonstrates |
|---------|---------------------|
| [hello-world](https://github.com/ironic-org/ironic/tree/main/examples/hello-world) | Minimal API with a controller, service, and JSON responses |
| [rest-api](https://github.com/ironic-org/ironic/tree/main/examples/rest-api) | Validation, versioning, serialization, compression, security, and testing |
| [todo-app](https://github.com/ironic-org/ironic/tree/main/examples/todo-app) | Full CRUD with SQL database, DTO validation, repositories, and services |
| [auth-api](https://github.com/ironic-org/ironic/tree/main/examples/auth-api) | JWT authentication, password hashing, guards, decorators, login/register/refresh |
| [blog-api](https://github.com/ironic-org/ironic/tree/main/examples/blog-api) | Cross-module DI, CRUD with categories, in-memory repositories, stats module, filtering, and slug management |

## Running an example

```bash
git clone https://github.com/ironic-org/ironic
cd ironic/examples/blog-api
SERVER_PORT=3002 cargo run
```

## hello-world

The simplest possible Ironic app — a single endpoint:

```rust
#[controller("/users")]
struct UsersController;

#[routes]
impl UsersController {
    #[get("/:id")]
    async fn get(&self, #[param] id: u64) -> Result<Json<UserView>, HttpError> {
        // ...
    }
}
```

## rest-api

A production-style API covering:

- Request validation with `garde` and `ValidationPipe`
- API versioning (URI prefix, headers, media types)
- Response serialization with role-based field rules
- Custom exception filters
- Compression (gzip, brotli, zstd)
- CORS and security headers
- Full test suite with `TestApplication`

## todo-app

A task management API demonstrating:

- SQL database integration with `sqlx`
- Repository pattern with PostgreSQL queries
- DTO validation with `garde`
- Full CRUD with `PUT`, `DELETE`, `POST`, `GET`
- Task completion toggle and bulk clear
- Tracing instrumentation with `#[instrument]`

## auth-api

An authentication API covering:

- JWT access + refresh token flow
- Argon2 password hashing via `PasswordService`
- Custom guards (`AuthGuard`) for protected routes
- Custom decorators (`current_user`) for extracting user context
- Role-based access with `Role` enum
- In-memory user store

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

- [x] Examples demonstrate real-world patterns
- [x] `hello-world` = minimal starting point
- [x] `rest-api` = production feature showcase
- [x] `todo-app` = database-driven CRUD
- [x] `auth-api` = authentication and authorization
- [x] `blog-api` = cross-module DI and sub-resource routing
