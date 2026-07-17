---
title: Custom Decorators
description: Create your own parameter extractors — extract user info, request metadata, or any custom data directly into handler arguments.
---

# Custom Decorators

## What is a Decorator?

A decorator is a **parameter extractor**. When a request arrives, the framework needs to know how to populate your handler's arguments:

```rust
async fn show(
    &self,
    #[param] id: Uuid,      // ← extracted from the URL path
    #[body] payload: Dto,    // ← extracted from the JSON body
    #[query] filter: String, // ← extracted from the query string
) -> Result<Json<Data>, HttpError> {
```

Each `#[...]` tells the framework **how** to get that argument from the request. Built-in decorators (`param`, `body`, `query`, `header`, `form`) cover common cases. A **custom decorator** lets you define your own extraction logic.

**Simple analogy:** Built-in decorators are like standard form fields — name, email, password. A custom decorator is like a derived field — you don't ask for "age", you compute it from the user's date of birth. It's data that comes from the request context, not a single header or query param.

## When to use a custom decorator

| Scenario | Example |
|---|---|
| Extract the current user from the auth context | `#[decorator(CurrentUser)]` |
| Parse pagination from query params | `#[decorator(Pagination)]` |
| Get the client's IP address | `#[decorator(ClientIp)]` |
| Read a value from request extensions | `#[decorator(RequestId)]` |
| Combine multiple headers into one struct | `#[decorator(DeviceInfo)]` |

## Creating a custom decorator

A decorator is a struct that implements `ParameterExtractor`:

```rust
use ironic::{ExtractFuture, ExtractedValue, ParameterExtractor, RequestContext};

pub struct CurrentUser;

impl ParameterExtractor for CurrentUser {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            // Read the user ID from request extensions (set by an auth guard)
            let user_id = context.extension::<String>().cloned();
            Ok(Box::new(user_id) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "current_user"
    }
}
```

The contract has two methods:
- `extract()` — async, takes `&mut RequestContext`, returns `Box<dyn Any + Send>` containing your data
- `description()` — a short diagnostic label

## Using it in a handler

Once defined, use `#[decorator(YourType)]` on a handler argument:

```rust
#[controller("/users")]
#[derive(Injectable)]
struct UserController;

#[routes]
impl UserController {
    #[get("/profile")]
    async fn profile(
        &self,
        #[decorator(CurrentUser)] user_id: Option<String>,
    ) -> Result<Json<User>, HttpError> {
        // user_id is extracted automatically
    }
}
```

The `#[decorator(Name)]` attribute tells the framework to call `Name::new()` and then `extract()` when resolving the argument.

## Complete example — Pagination decorator

Extract `?page=1&size=20` from query params into a typed struct:

```rust
use ironic::{ExtractFuture, ExtractedValue, ParameterExtractor, RequestContext};

#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub page: u64,
    pub size: u64,
}

pub struct Pagination;

impl Pagination {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ParameterExtractor for Pagination {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let query = context.request().uri().query().unwrap_or_default();
            let page = get_param(query, "page").unwrap_or(1);
            let size = get_param(query, "size").unwrap_or(20).min(100);
            Ok(Box::new(PaginationParams { page, size }) as ExtractedValue)
        })
    }

    fn description(&self) -> &'static str {
        "pagination"
    }
}

fn get_param(query: &str, key: &str) -> Option<u64> {
    let prefix = format!("{key}=");
    query.split('&').find_map(|pair| {
        pair.starts_with(&prefix).then(|| pair[prefix.len()..].parse().ok())?
    })
}
```

Usage:

```rust
#[get("/")]
async fn list(
    &self,
    #[decorator(Pagination)] pagination: PaginationParams,
) -> Result<Json<Vec<Item>>, HttpError> {
    let offset = (pagination.page - 1) * pagination.size;
    // ...fetch page from database...
}
```

## The `create_param_decorator!` macro

For reusable decorators, use the macro to create a type alias:

```rust
use ironic::create_param_decorator;

// Define the extractor
pub struct ClientIp;
impl ParameterExtractor for ClientIp { /* ... */ }

// Create a type alias so #[decorator(client_ip)] works
create_param_decorator!(client_ip, ClientIp);
```

This lets you use `#[decorator(client_ip)]` instead of `#[decorator(ClientIp)]` — useful when you prefer snake_case naming in your route signatures.

## When NOT to use a decorator

| Situation | Use instead |
|---|---|
| Simple single-value extraction | `#[header]`, `#[param]`, `#[query]` |
| Validation/transformation after extraction | `#[pipe]` |
| Logic that affects the response (not just parameters) | `#[interceptor]` |
| Auth/permission checks | `#[guard]` |

Decorators are specifically for **extracting handler arguments**. If you need to modify the response or block the request, use another tool.

## Common mistakes

| Mistake | Fix |
|---|---|
| Decorator struct missing `::new()` | `#[decorator(Name)]` generates `Name::new()` — add a `pub const fn new()` constructor |
| Returning wrong type from extract | Cast to `ExtractedValue`: `Ok(Box::new(value) as ExtractedValue)` |
| Making extract synchronous | The trait requires `async` — use `Box::pin(async move { ... })` |
| Over-engineering | Most extraction needs are covered by `#[param]`, `#[body]`, `#[query]`, `#[header]` |

## What you learned

- [x] Decorators are custom parameter extractors for handler arguments
- [x] Implement `ParameterExtractor` with `extract()` and `description()`
- [x] Use with `#[decorator(YourType)]` in handler signatures
- [x] `ExtractedValue` is `Box<dyn Any + Send>` — cast with `as`
- [x] Use `create_param_decorator!` for snake_case alias names
