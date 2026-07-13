---
title: Authentication and authorization
description: Password hashing, bearer JWTs, OAuth 2.0, sessions, roles, and permissions in Ironic.
---

# Authentication and authorization

Authentication support is feature-gated so applications only compile the protocols they use:

```toml
[dependencies]
ironic = { version = "0.1", features = ["authentication"] }
```

Use `auth` for Argon2 password hashing and pipeline contracts, `jwt` for signed bearer tokens,
`oauth` for OAuth 2.0 Authorization Code flows, or `sessions` for the session-store contract.
`authentication` enables all four.

## Passwords

```rust
use ironic::auth::{hash_password, verify_password};

# fn example() -> Result<(), ironic::auth::password_driver::password_hash::Error> {
let encoded = hash_password(b"user supplied password")?;
assert!(verify_password(b"user supplied password", &encoded)?);
# Ok(())
# }
```

Store only the encoded Argon2id hash. Never log a password or include it in an error response.

## Request authentication

Implement `Authenticator<MyPrincipal>` and register
`AuthenticationMiddleware::<_, MyPrincipal>::new(authenticator)`. The middleware stores a typed
`AuthContext<MyPrincipal>` in `RequestContext`. Anonymous requests continue through the pipeline;
add `RequireAuthenticated<MyPrincipal>` where authentication is mandatory.

Principals implement `Principal`. Implement `Authorizable` to use role and permission guards:

```text
PipelineComponents::new()
    .guard(RequireAccess::<User>::role("admin"))
```

## JWT bearer tokens

```rust
use ironic::auth::jwt::{JwtService, driver::Algorithm};

let service = JwtService::hmac(
    std::env::var("JWT_SECRET").unwrap().as_bytes(),
    Algorithm::HS256,
);
```

Configure issuer, audience, required claims, and clock-skew policy through `validation_mut`. Use
`JwtBearerAuthenticator` to validate an Authorization header and map claims to your principal.
Ironic never accepts an algorithm from unverified token data; the configured validation policy is
authoritative.

## OAuth 2.0

`oauth::basic_client` configures the authorization, token, and redirect endpoints.
`oauth::authorization_request` creates fresh CSRF state and an S256 PKCE challenge. Persist the
returned state and verifier in the user's session, compare callback state before exchanging the
code, and configure the upstream HTTP client to reject redirects to reduce SSRF exposure.

## Sessions

`SessionStore` is the backend boundary. `InMemorySessionStore` is suitable for development and
single-process services; production applications should implement the same trait over Redis or a
database. Session IDs contain 256 bits from the operating-system random source and their `Debug`
output is redacted. `session_cookie` creates an HttpOnly, SameSite=Lax cookie; set `secure = true`
outside local HTTP development.
