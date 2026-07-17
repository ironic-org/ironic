---
title: Custom Decorators
description: Create your own parameter extractors — extract the current user, request metadata, or any custom data from requests.
---

# Custom Decorators

## What you'll learn

- Create custom parameter extractors
- Use them in route handlers with `#[decorator(Name)]`
- Extract common data (current user, client IP, locale) once and reuse

Enable in `Cargo.toml`:

```toml
ironic = { features = ["custom-decorators"] }
```

---

## Creating a custom decorator

```rust
use ironic::{ExtractFuture, ParameterExtractor, RequestContext, create_param_decorator};

struct CurrentUser;

impl ParameterExtractor for CurrentUser {
    fn extract<'a>(&'a self, context: &'a mut RequestContext) -> ExtractFuture<'a> {
        Box::pin(async move {
            // Read the user from the request context
            let user_id = context.get::<u64>("user_id").copied();
            Ok(Box::new(user_id))
        })
    }

    fn description(&self) -> &'static str {
        "current_user"
    }
}

create_param_decorator!(current_user, CurrentUser);
```

## Using it in a handler

```rust
#[routes]
impl Controller {
    #[get("/me")]
    async fn profile(
        &self,
        #[decorator(current_user)] user_id: Option<u64>,  // ← Extracted automatically
    ) -> Result<Json<User>, HttpError> {
        match user_id {
            Some(id) => self.service.find_user(id).map(Json),
            None => Err(HttpError::unauthorized("UNAUTHORIZED", "Login required")),
        }
    }
}
```

## Common custom decorators

| Decorator | What it extracts |
|-----------|-----------------|
| `CurrentUser` | Authenticated user ID |
| `ClientIp` | Client's IP address |
| `UserLocale` | Accept-Language header |
| `RequestId` | X-Request-ID header |

## What you learned

- [x] Implement `ParameterExtractor` to extract custom data
- [x] Register with `create_param_decorator!`
- [x] Use with `#[decorator(name)]` in route handlers
- [x] Extract once, reuse across many handlers
