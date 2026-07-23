---
title: Guards
description: Protect routes with authorization guards — authenticate users, check roles, and control access.
---

# Guards

Guards control whether a request can reach a route handler. They run after routing but before the handler, making them the right place for authorization logic.

## How guards work

```
Request → Router → Guard → Handler
                     │
                     ▼
                  Deny → 401/403 response
```

A guard receives the request context and returns a decision:

```rust
pub trait Guard {
    type Error: Into<HttpError>;
    async fn decide(&self, ctx: &GuardContext) -> GuardDecision<Self::Error>;
}
```

## Guard decisions

| Decision | Result |
|----------|--------|
| `GuardDecision::Allow` | Request proceeds to the handler |
| `GuardDecision::Deny(error)` | Request is rejected with the error |

## Writing a guard

```rust
struct AuthGuard;

impl Guard for AuthGuard {
    type Error = HttpError;

    async fn decide(&self, ctx: &GuardContext) -> GuardDecision<Self::Error> {
        let token = match ctx.request.headers().get("Authorization") {
            Some(v) => v.to_str().unwrap_or(""),
            None => return GuardDecision::Deny(HttpError::unauthorized(
                "AUTH_REQUIRED", "Authorization header is missing",
            )),
        };

        if token.starts_with("Bearer ") {
            GuardDecision::Allow
        } else {
            GuardDecision::Deny(HttpError::unauthorized(
                "AUTH_INVALID", "Invalid authorization format",
            ))
        }
    }
}
```

## Applying guards

### On a controller method

```rust
#[routes]
impl UsersController {
    #[guard(AuthGuard)]
    #[get("/profile")]
    async fn get_profile(&self) -> Json<User> {
        // Only authenticated users can access this
    }
}
```

### On a controller (all routes)

```rust
#[controller("/admin")]
#[guard(AdminGuard)]
struct AdminController {
    service: Arc<AdminService>,
}
```

## Built-in guards

Ironic provides several built-in guards:

| Guard | Description |
|-------|-------------|
| `AuthenticationGuard` | Requires a valid authentication token |
| `RoleGuard` | Requires a specific user role |
| `PermissionGuard` | Requires a specific permission |

## Combining guards

Multiple guards can be applied to the same route. They run in order:

```rust
#[guard(AuthGuard)]
#[guard(RoleGuard::new("admin"))]
#[get("/admin/users")]
async fn admin_list(&self) -> Json<Vec<User>> {
    // Must be authenticated AND have admin role
}
```

## Guard context

The `GuardContext` provides access to:

- The HTTP request (headers, method, URI)
- The DI container (for resolving services)
- Route metadata (controller, method name)

```rust
async fn decide(&self, ctx: &GuardContext) -> GuardDecision<Self::Error> {
    let config = ctx.container.resolve::<AppConfig>().await.map_err(|e|
        GuardDecision::Deny(HttpError::internal("CONFIG_ERROR", "config not available"))
    )?;
    // Use config to make authorization decision
}
```

## Testing guards

```rust
#[tokio::test]
async fn test_auth_guard() {
    let guard = AuthGuard;
    let ctx = GuardContext::new(request_with_token("valid_token"));

    let decision = guard.decide(&ctx).await;
    assert!(matches!(decision, GuardDecision::Allow));
}
```
