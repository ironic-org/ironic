---
title: Authentication
description: Add login to your API — JWT tokens, OAuth2 social login, session management, and password hashing.
---

# Authentication

## What you'll learn

- Hash passwords securely with Argon2id
- Issue and verify JWT tokens
- Add OAuth2 login (Google, GitHub, etc.)
- Manage user sessions
- Protect routes with auth guards

Enable in `Cargo.toml`:

```toml
ironic = { features = ["authentication"] }
# Or pick individual features:
# ironic = { features = ["auth", "jwt"] }     ← Passwords + JWT only
# ironic = { features = ["auth", "oauth"] }    ← Passwords + OAuth only
# ironic = { features = ["auth", "sessions"] } ← Passwords + sessions
```

---

## 1. Password hashing (Argon2id)

Never store raw passwords. Hash them:

```rust
use ironic::auth::{hash_password, verify_password};

let hash = hash_password(b"my-secret-password")?;

// Verify later:
assert!(verify_password(b"my-secret-password", &hash)?);
assert!(!verify_password(b"wrong-password", &hash)?);
```

> Argon2id is the **current best practice** for password hashing — it's memory-hard, resistant to GPU attacks, and recommended by OWASP.

## 2. JWT tokens

Issue tokens when users log in, verify them on every request:

```rust
use ironic::auth::jwt::{Claims, JwtService};
use std::time::Duration;

#[derive(Injectable)]
pub struct AuthService {
    jwt: std::sync::Arc<JwtService>,
}

impl AuthService {
    pub fn login(&self, user_id: u64) -> Result<String, HttpError> {
        let claims = Claims::new()
            .with_subject(&user_id.to_string())
            .with_expiry(Duration::from_secs(3600));  // 1 hour

        self.jwt.encode(&claims)
            .map_err(|e| HttpError::internal("JWT_ERROR", e.to_string()))
    }

    pub fn verify(&self, token: &str) -> Result<u64, HttpError> {
        let claims = self.jwt.decode(token)
            .map_err(|_| HttpError::unauthorized("INVALID_TOKEN", "Token is invalid or expired"))?;
        claims.subject.parse()
            .map_err(|_| HttpError::unauthorized("INVALID_TOKEN", "Invalid subject"))
    }
}
```

### Protecting routes

```rust
// In your controller:
#[get("/me")]
async fn profile(
    &self,
    #[header("Authorization")] auth: String,
) -> Result<Json<UserView>, HttpError> {
    let token = auth.trim_start_matches("Bearer ");
    let user_id = self.auth.verify(token)?;
    let user = self.users.find_by_id(user_id)?;
    Ok(Json(user.into()))
}
```

## 3. OAuth2 (social login)

Add "Log in with Google/GitHub":

```rust
use ironic::auth::oauth::{OAuthClient, OAuthConfig};

let config = OAuthConfig::new()
    .client_id("your-client-id")
    .client_secret("your-client-secret")
    .redirect_url("http://localhost:3000/auth/callback")
    .auth_url("https://accounts.google.com/o/oauth2/auth")
    .token_url("https://oauth2.googleapis.com/token");

let client = OAuthClient::new(config);

// Step 1: Redirect user to Google for login
let auth_url = client.authorize_url(&["email", "profile"]);

// Step 2: Exchange code for token after callback
let token = client.exchange_code(code).await?;

// Step 3: Fetch user info
let user_info = client.fetch_user_info(&token).await?;
```

## 4. Sessions

For traditional server-rendered apps, use sessions:

```rust
use ironic::auth::sessions::{InMemorySessionStore, SessionStore};

let store = InMemorySessionStore::new();

// Create session
let session_id = store.create(user_id, Duration::from_secs(3600))?;

// Get user from session
let user_id = store.get(&session_id)?;

// Destroy session (logout)
store.destroy(&session_id)?;
```

## Authentication flow

```
POST /login { email, password }
    │
    ▼
AuthService.login()
    │
    ├── Verify password hash
    ├── Create JWT token
    └── Return token to client
          │
          ▼
Client sends: Authorization: Bearer <token>
          │
          ▼
Controller: verify(token) → user_id → do work
```

## Try it yourself

1. Add a `POST /register` route that hashes a password
2. Add a `POST /login` route that returns a JWT
3. Protect `GET /me` so it requires a valid JWT
4. Test: call `/me` without token → 401. Call with token → 200.

## What you learned

- [x] Hash passwords with Argon2id (never store plain text)
- [x] Issue and verify JWT tokens
- [x] Add OAuth2 for social login
- [x] Use sessions for traditional apps
- [x] Protect routes with auth checks
