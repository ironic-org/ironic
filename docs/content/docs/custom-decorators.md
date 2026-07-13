---
title: Custom decorators
description: Define reusable parameter extractors with the create_param_decorator macro and pipe chaining.
---

# Custom decorators

Use `create_param_decorator!` to define named parameter decorators that extract typed values from
requests. Combine them with pipes for validation and transformation.

```toml
ironic = { features = ["custom-decorators"] }
```

## Defining a custom decorator

Implement `ParameterExtractor` and register it as a named decorator:

```rust
use ironic::{ParameterExtractor, RequestContext, ExtractFuture, create_param_decorator, HttpError};

struct CurrentUser;

impl ParameterExtractor for CurrentUser {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            let user = context.extension::<AuthUser>()
                .ok_or_else(|| HttpError::unauthorized("UNAUTHORIZED", "not authenticated"))?;
            Ok(Box::new(user.clone()) as ExtractedValue)
        })
    }
    fn description(&self) -> &'static str {
        "current_user"
    }
}

create_param_decorator!(current_user, CurrentUser);
```

## Using in route handlers

```rust
#[routes]
impl UserController {
    #[get("/me")]
    async fn profile(&self, #[custom(current_user)] user: AuthUser) -> Result<impl IntoFrameworkResponse, HttpError> {
        // `user` is extracted by the CurrentUser extractor
    }
}
```

The decorator name after `#[custom(...)]` must match the name passed to `create_param_decorator!`.

## Chaining pipes

Custom decorators support pipe chaining just like built-in extractors:

```rust
#[get("/item/:id")]
async fn get(
    &self,
    #[custom(header_id)] #[pipe(parse_int)] id: i64,
) -> Result<impl IntoFrameworkResponse, HttpError> { ... }
```

Pipes transform the decorator's output before the handler receives it.

## Extraction context

The `extract` method receives `&mut RequestContext`, granting access to:

- The raw request (method, URI, headers, body, path parameters)
- Request extensions (typed request-scoped state set by middleware or guards)

Return `HttpError` to short-circuit the request. The error propagates through the normal
pipeline and exception filter chain.
