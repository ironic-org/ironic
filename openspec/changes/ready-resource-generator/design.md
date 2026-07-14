## Architecture

### New CLI command

```
ironic generate ready-resource auth    → full auth (passwords + JWT + OAuth + sessions + RBAC)
ironic generate ready-resource auth-basic  → passwords + sessions only
ironic generate ready-resource auth-jwt    → JWT tokens only
ironic generate ready-resource auth-oauth  → OAuth social login only
```

### Generated module structure

```
src/modules/auth/
├── mod.rs                          ← AuthModule wiring (imports UserModule)
├── controller/
│   ├── mod.rs
│   └── auth_controller.rs          ← POST /auth/register, /auth/login, /auth/refresh
│                                      GET /auth/me, /auth/logout
│                                      GET /auth/oauth/google, /auth/oauth/github
│                                      GET /auth/oauth/callback
├── services/
│   ├── mod.rs
│   ├── auth_service.rs             ← register(), login(), refresh(), verify(), oauth_login()
│   └── password_service.rs         ← hash(), verify() using Argon2id
├── guards/
│   ├── mod.rs
│   ├── auth_guard.rs               ← extracts JWT from Authorization header
│   └── role_guard.rs               ← checks user role against required roles
├── dto/
│   ├── mod.rs
│   ├── register_dto.rs             ← email, password, name
│   ├── login_dto.rs                ← email, password
│   ├── refresh_dto.rs              ← refresh_token
│   └── token_response.rs           ← access_token, refresh_token, expires_in
├── entities/
│   ├── mod.rs
│   ├── user.rs                     ← id, email, password_hash, name, role, provider
│   └── role.rs                     ← Admin, User, Moderator enum
├── decorators/
│   ├── mod.rs
│   ├── current_user.rs             ← extracts authenticated user from request
│   └── roles.rs                    ← role-based route decorator
├── tests/
│   ├── mod.rs
│   ├── unit/
│   │   ├── auth_service_test.rs
│   │   ├── password_service_test.rs
│   │   └── guard_test.rs
│   └── integration/
│       ├── register_test.rs
│       ├── login_test.rs
│       └── auth_flow_test.rs       ← full register → login → me → refresh → logout cycle
```

### Implementation approach

1. **Generator logic** — New `ready_resource.rs` file in `crates/ironic-cli/src/generators/` with:
   - `generate_ready_resource(root, name, variant)` entry point
   - Template functions for each generated file
   - Variant selection (full/basic/jwt/oauth)

2. **CLI wiring** — Add `ReadyResource` variant to `Generator` enum in `cli.rs`, dispatch in `generate.rs`

3. **Template approach** — Use Rust `format!()` for simpler templates (like existing generators) but with richer content for auth-specific logic

4. **Zero-config** — Generated module compiles and works immediately. JWT secret defaults to an env var `JWT_SECRET` (without it, uses a development-only fallback that prints a warning).

### Generated API endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/register` | No | Create account |
| POST | `/auth/login` | No | Get JWT tokens |
| POST | `/auth/refresh` | Refresh token | Get new access token |
| GET | `/auth/me` | JWT | Get current user |
| POST | `/auth/logout` | JWT | Invalidate token |
| GET | `/auth/oauth/google` | No | Start Google OAuth flow |
| GET | `/auth/oauth/github` | No | Start GitHub OAuth flow |
| GET | `/auth/oauth/callback` | OAuth state | Complete OAuth flow |

### Dependencies added to generated project

```toml
jsonwebtoken = "9"
argon2 = "0.5"
oauth2 = "5.0"       # only for auth-oauth and auth variants
getrandom = "0.4"    # for session IDs
```

### Security considerations

- Passwords hashed with Argon2id (memory-hard, GPU-resistant)
- JWT tokens signed with HS256 by default, configurable to RS256
- Access tokens short-lived (15 min default), refresh tokens long-lived (7 days)
- Rate limiting applied to login/register endpoints in the generated code
- OAuth state parameter with PKCE for social login
- `Secret<T>` wrapper used for JWT secret and OAuth client secrets
