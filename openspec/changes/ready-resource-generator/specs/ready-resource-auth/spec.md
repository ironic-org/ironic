# Ready Resource Auth

The `ironic generate ready-resource auth` command generates a complete, production-grade authentication module.

## Requirements

### Generator Command

- **RRA-001**: The CLI SHALL accept `ironic generate ready-resource auth` as a valid command
- **RRA-002**: The CLI SHALL accept `ironic generate ready-resource auth-basic`, `auth-jwt`, and `auth-oauth` as variant subcommands
- **RRA-003**: Generating into an existing project SHALL auto-register the module in `modules/mod.rs` and `app.rs`

### Generated Files

- **RRA-010**: The generator SHALL create an `AuthService` with `register()`, `login()`, `refresh()`, `me()`, and `logout()` methods
- **RRA-011**: The generator SHALL create a `PasswordService` with `hash()` and `verify()` using Argon2id
- **RRA-012**: The generator SHALL create an `AuthController` at `/auth` with routes for register, login, refresh, me, logout, and OAuth callbacks
- **RRA-013**: The generator SHALL create a `User` entity with fields: id, email, password_hash, name, role, provider, created_at
- **RRA-014**: The generator SHALL create a `Role` enum with variants: Admin, User, Moderator
- **RRA-015**: The generator SHALL create role-based guards that protect routes using `#[use_guard(RoleGuard::new(&["admin"]))]`
- **RRA-016**: The generator SHALL create a custom decorator `current_user` that extracts the authenticated user from the request

### Auth Variants

- **RRA-020**: `auth` variant SHALL generate all features: passwords, JWT, OAuth, sessions, RBAC
- **RRA-021**: `auth-basic` variant SHALL generate only: passwords and sessions
- **RRA-022**: `auth-jwt` variant SHALL generate only: JWT tokens and password hashing
- **RRA-023**: `auth-oauth` variant SHALL generate only: OAuth2 social login with Google and GitHub providers

### Security

- **RRA-030**: Passwords SHALL be hashed with Argon2id (memory-hard, GPU-resistant)
- **RRA-031**: JWT tokens SHALL use HS256 by default, with RS256 configurable via env var
- **RRA-032**: Access tokens SHALL be short-lived (15 min default), refresh tokens long-lived (7 days)
- **RRA-033**: OAuth flows SHALL include state parameter with PKCE
- **RRA-034**: JWT secret SHALL be loaded from `JWT_SECRET` environment variable
- **RRA-035**: Generated code SHALL use `Secret<T>` for JWT secret and OAuth client secrets

### Testing

- **RRA-040**: Generated module SHALL include unit tests for AuthService (register, login, refresh flow)
- **RRA-041**: Generated module SHALL include integration tests via TestApplication (full HTTP flow)
- **RRA-042**: Test suite SHALL pass without external dependencies (no database, no live OAuth)

### Generated Cargo.toml Dependencies

- **RRA-050**: Generated project SHALL include `jsonwebtoken = "9"` for JWT variants
- **RRA-051**: Generated project SHALL include `argon2 = "0.5"` for password variants
- **RRA-052**: Generated project SHALL include `oauth2 = "5.0"` for OAuth variants
- **RRA-053**: Generated project SHALL include `getrandom = "0.4"` for session variants
