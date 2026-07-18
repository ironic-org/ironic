---
title: JWT Auth in 5 Minutes
description: Add JWT authentication to any Ironic API with a single #[jwt_guard] declaration — no manual guard wiring required.
---

# JWT Auth in 5 Minutes

## What you'll learn

- Go from zero to protected routes in minutes using `#[jwt_guard]`
- Understand what the macro generates (claims, principal, authenticator, guard, middleware)
- Customize claims and principal mapping for your domain

---

## Quick Start

Add `jwt` and `auth` to your `Cargo.toml`:

```toml
ironic = { features = ["jwt", "auth", "sqlx-postgres"] }
```

Define your auth configuration in a single macro invocation:

```rust
use ironic::jwt_guard;

#[jwt_guard(
    secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
    claims = UserClaims { sub: String, exp: u64, iat: u64 },
    principal = User { id: String },
    map = |c: UserClaims| -> Result<User, ironic::auth::AuthError> {
        Ok(User { id: c.sub })
    }
)]
pub struct Auth;
```

Register the middleware in your `Application::builder()`:

```rust
let app = Application::builder()
    .module(AppModule::definition())
    .middleware(Auth::auth_middleware())
    .platform(AxumAdapter::new())
    .build()
    .await?;
```

Protect any controller or route:

```rust
#[controller("/api/blogs")]
#[guard(Auth::AuthGuard)]
#[derive(Injectable)]
pub struct BlogsController { ... }
```

---

## What `#[jwt_guard]` Generates

The macro expands into five components:

### 1. Claims Struct

```rust
#[derive(Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}
```

### 2. Principal Struct

```rust
pub struct User {
    pub id: String,
}

impl ironic::auth::Principal for User {
    fn subject(&self) -> &str {
        &self.id
    }
}
```

### 3. Guard Type Alias

```rust
pub type AuthGuard = ironic::auth::RequireAuthenticated<User>;
```

Use `#[guard(Auth::AuthGuard)]` on controllers or individual routes.

### 4. Middleware Constructor

```rust
pub fn auth_middleware()
    -> ironic::auth::AuthenticationMiddleware<...>
```

Pass to `Application::builder().middleware(Auth::auth_middleware())`.

### 5. Config Struct

Your original struct is preserved. You can add methods to it, store it in DI, or hold references to the JWT service.

---

## Role-Based Access Control

For role/permission checks, extend `User` to implement `Authorizable`:

```rust
impl ironic::auth::Authorizable for User {
    fn has_role(&self, role: &str) -> bool {
        // Look up roles from your database
        matches!(role, "admin" | "editor")
    }

    fn has_permission(&self, permission: &str) -> bool {
        // Look up permissions from your database
        !permission.contains("delete")
    }
}
```

Then use `RequireAccess` guards:

```rust
#[controller("/api/admin")]
#[guard(ironic::auth::RequireAccess::<User>::role("admin"))]
#[derive(Injectable)]
pub struct AdminController { ... }
```

---

## Manual Claims Extraction

If you need the raw claims inside a handler, access them from the request context:

```rust
#[get("/me")]
async fn me(&self, context: &RequestContext) -> Response {
    let ctx = context.extension::<AuthContext<User>>()
        .ok_or_else(|| HttpError::unauthorized("NOT_AUTHENTICATED", "Not authenticated"))?;

    let user = ctx.principal()
        .ok_or_else(|| HttpError::unauthorized("NOT_AUTHENTICATED", "No principal"))?;

    Response::json(200, &json!({ "user_id": user.subject() }))
}
```

---

## Feature Flags

| Feature | Required |
|---------|----------|
| `jwt` | Required (enables `auth`, `jsonwebtoken`, and `jwt_guard` macro) |
| `auth` | Implied by `jwt` |
