---
title: Authentication Middleware
description: Authenticate requests with AuthenticationMiddleware — extract principals, protect routes with guards.
---

# Authentication Middleware

`AuthenticationMiddleware` extracts an authenticated principal from every request and stores it in the request context. Guards then check whether the principal is allowed to access a route.

## How it works

1. `AuthenticationMiddleware` runs on every request.
2. It calls an `Authenticator` that you provide (JWT, session, API key, etc.).
3. The result — an `AuthContext<P>` — is stored in request extensions.
4. Guards like `RequireAuthenticated<P>` and `RequireAccess<P>` check the context and reject unauthenticated or unauthorized requests.

```
REQUEST IN → AuthenticationMiddleware → RequireAuthenticated → RequireAccess → HANDLER
                    ↓                        ↓                    ↓
           calls Authenticator        checks principal     checks permission
           stores AuthContext<P>      returns 401 if none  returns 403 if denied
```

## Enabling

```toml
ironic = { features = ["auth"] }
```

## How to use

Define your principal type and an authenticator, then register the middleware and guards:

```rust
use ironic::prelude::*;
use ironic::auth::{AuthenticationMiddleware, Authenticator, AuthContext};
use ironic::auth::guards::{RequireAuthenticated, RequireAccess};

// Your application user
#[derive(Clone, Debug)]
struct User {
    id: u64,
    email: String,
    role: String,
}

// Implement Authorizable so RequireAccess can check permissions
impl Authorizable for User {
    fn has_access(&self, permission: &str) -> bool {
        match permission {
            "admin" => self.role == "admin",
            "write" => self.role == "admin" || self.role == "editor",
            _ => false,
        }
    }
}

// Register middleware + guards
Application::builder()
    .module(AppModule::definition())
    .middleware(AuthenticationMiddleware::new(MyAuthenticator))
    .platform(AxumAdapter::new())
    .build().await.unwrap();

// Protect routes
#[controller("/admin")]
#[middleware(AuthenticationMiddleware::new(MyAuthenticator))]
#[guard(RequireAuthenticated::<User>::new())]
struct AdminController;
```

## Accessing the authenticated user in handlers

```rust
fn profile(context: RequestContext) -> impl IntoResponse {
    let auth = context.extension::<AuthContext<User>>().unwrap();
    match auth.principal() {
        Some(user) => format!("Hello, {}", user.email),
        None => "anonymous".to_string(),
    }
}
```

## Guards reference

| Guard | What it does | HTTP status |
|---|---|---|
| `RequireAuthenticated<P>` | Rejects requests with no authenticated principal | 401 Unauthorized |
| `RequireAccess<P>` | Rejects requests where `P::has_access()` returns false | 403 Forbidden |

`RequireAccess` requires your principal type to implement `Authorizable`.

## Pipeline order

Middleware runs before guards. `AuthenticationMiddleware` must be registered before any auth guard:

```rust
// ✅ Correct — middleware before guards
Application::builder()
    .module(AppModule::definition())
    .middleware(AuthenticationMiddleware::new(MyAuthenticator))
    .platform(AxumAdapter::new())
    .build().await.unwrap();

// Route-level guard
#[controller("/admin")]
struct AdminController;

#[routes]
impl AdminController {
    #[get("/profile")]
    #[guard(RequireAuthenticated::<User>::new())]
    async fn profile(&self) -> Result<Json<User>, HttpError> {
        // ...
    }
}
```
