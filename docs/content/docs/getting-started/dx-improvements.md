---
title: Developer Experience Improvements
description: Higher-level APIs that eliminate boilerplate — FromRow, jwt_guard, config presets, pagination, guard_fn, Merge, and more.
---

# Developer Experience

Starting a new project with Ironic means writing less boilerplate code than ever before. Here are the higher-level APIs designed to let you focus on business logic.

---

## Database

### [Auto Row Mapping](/docs/data-auth/from-row#derivefromrow--automatic-row-mapping)

```rust
// Before: 6 manual r.get() calls per query
let post = BlogPost {
    id: row.get("id"),
    title: row.get("title"),
    // ...
};

// After: one derive, zero hand-written mapping
#[derive(FromRow)]
pub struct BlogPost { pub id: Uuid, pub title: String }
```

### [Error Conversion](/docs/data-auth/from-row#sqlxerrorext--automatic-error-mapping)

```rust
// Before: every query has a manual map_err
.map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?

// After: consistent error codes in one call
.map_db_err("BLOG_POST", "FIND")?
```

### [Async Database Startup](/docs/data-auth/from-row#asyncmoduleinit--async-database-startup)

```rust
// Before: rt.block_on() hack in sync Module::definition()
// After: just implement AsyncModuleInit — runs during build()
#[module(async_init = [DatabaseModule])]
```

---

## Authentication

### [JWT in 5 Minutes](/docs/data-auth/quick-auth)

```rust
// Before: 172 lines of manual JWT code
// After: one macro declaration
#[jwt_guard(
    secret = std::env::var("JWT_SECRET")?,
    claims = UserClaims { sub: String, exp: u64 },
    principal = User { id: String },
    map = |c: UserClaims| -> Result<User, AuthError> { Ok(User { id: c.sub }) }
)]
pub struct Auth;

// Use it:
app.middleware(Auth::auth_middleware());
#[guard(Auth::AuthGuard)]
```

---

## Configuration

### [Batteries-Included Presets](/docs/core/configuration)

```rust
// DatabaseConfig, AuthConfig, ServerConfig, RedisConfig are ready to use.
// Just embed them in your AppConfig:

#[derive(Deserialize)]
pub struct AppConfig {
    pub database: ironic::config::presets::DatabaseConfig,
    pub auth: ironic::config::presets::AuthConfig,
    pub server: ironic::config::presets::ServerConfig,
}
```

Secrets are automatically redacted: `#[derive(Debug)]` prints `[REDACTED]`.

---

## Pagination

### [Built-in Extractor](/docs/http-api/pagination-extractor)

```rust
// Before: manual query string parsing
// After: one decorator, zero parsing code
#[get("")]
async fn list(&self, #[decorator(Pagination)] p: Pagination) -> Response {
    let items = self.service.list(p.offset(), p.limit()).await?;
    Response::json(200, &items)
}
```

---

## Structural Updates

### [Merge Derive](/docs/core/macros)

```rust
// Before: five if-let-some clone assignments
let title = dto.title.clone().unwrap_or_else(|| current.title.clone());

// After: one derive, zero manual merging
#[derive(Merge)]
pub struct UpdateBlogDto { pub title: Option<String>, pub content: Option<String> }

dto.merge_into(&mut post);
```

---

## Trait Helpers

### [guard_fn! and intercept_fn!](/docs/core/macros)

```rust
// Before: Box::pin(async move { ... })
impl Guard for MyGuard {
    fn can_activate(&self, context: &mut RequestContext) -> GuardFuture {
        guard_fn!(context, {
            // your logic here
            GuardDecision::Allow
        })
    }
}
```

---

## OpenAPI

### [Smarter Schema Derive](/docs/http-api/openapi)

The `#[derive(OpenApiSchema)]` now reads:

| Attribute | Schema Effect |
|-----------|--------------|
| `src(skip)` | Exclude field from schema |
| `src(default)` | Remove from required array |
| `#[garde(length(min = 1, max = 255))]` | Add `minLength` / `maxLength` |
| `#[garde(range(min = 0, max = 100))]` | Add `minimum` / `maximum` |
| `#[garde(email)]` | Add `format: "email"` |
| `#[garde(url)]` | Add `format: "uri"` |
| `#[garde(pattern("..."))]` | Add `pattern` constraint |

No more manual `impl OpenApiSchema` with hardcoded JSON.

---

## Summary

| Task | Before | After |
|------|--------|-------|
| Map query results to structs | `r.get("field")` × N fields | `#[derive(FromRow)]` |
| Convert DB errors to HTTP | `.map_err(internal(..))?` | `.map_db_err("ENTITY", "OP")?` |
| Connect to DB at startup | `rt.block_on()` hack | `AsyncModuleInit` |
| Add JWT auth | 172 lines manual | `#[jwt_guard]` (30 lines) |
| Parse pagination params | 20 lines manual parsing | `#[decorator(Pagination)]` |
| Merge update DTOs | 5+ clone/if-let blocks | `#[derive(Merge)]` |
| Write Box::pin boilerplate | Every trait impl | `guard_fn!` / `intercept_fn!` |
