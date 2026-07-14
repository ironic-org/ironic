# Tasks

## 1. CLI Wiring

- [ ] 1.1 Add `ReadyResource` variant to `Generator` enum in `crates/ironic-cli/src/cli.rs` with `auth`, `auth-basic`, `auth-jwt`, `auth-oauth` sub-variants
- [ ] 1.2 Add dispatch case in `crates/ironic-cli/src/commands/generate.rs` for `Generator::ReadyResource`
- [ ] 1.3 Create `crates/ironic-cli/src/generators/ready_resource.rs` with `generate_ready_resource()` entry point

## 2. Auth Module Templates

- [ ] 2.1 Create template for `User` entity (`id`, `email`, `password_hash`, `name`, `role`, `provider`, `created_at`)
- [ ] 2.2 Create template for `Role` enum (`Admin`, `User`, `Moderator`)
- [ ] 2.3 Create template for `PasswordService` with `hash()` and `verify()` using Argon2id
- [ ] 2.4 Create template for `AuthService` with `register()`, `login()`, `refresh()`, `me()`, `logout()`, `oauth_login()`
- [ ] 2.5 Create template for `AuthController` at `/auth` with all routes
- [ ] 2.6 Create templates for DTOs: `RegisterDto`, `LoginDto`, `RefreshDto`, `TokenResponse`
- [ ] 2.7 Create template for JWT token handling (issue, verify, refresh)
- [ ] 2.8 Create template for OAuth2 flows (Google, GitHub) with PKCE
- [ ] 2.9 Create template for session management (`InMemorySessionStore`)

## 3. Guards & Decorators

- [ ] 3.1 Create template for `AuthGuard` — extracts JWT from Authorization header, sets current user in request context
- [ ] 3.2 Create template for `RoleGuard` — checks user role against required roles
- [ ] 3.3 Create template for `current_user` custom decorator using `create_param_decorator!`
- [ ] 3.4 Create template for `roles` custom decorator

## 4. Module Wiring

- [ ] 4.1 Create template for `AuthModule` (or variant-specific module) with providers, controllers, guards, exports
- [ ] 4.2 Auto-register module in `src/modules/mod.rs` via `ensure_items()`
- [ ] 4.3 Auto-import module in `src/app.rs` via `ensure_module_import()`

## 5. Test Templates

- [ ] 5.1 Create unit tests for `PasswordService` (hash, verify, unique salts)
- [ ] 5.2 Create unit tests for `AuthService` (register, login, refresh)
- [ ] 5.3 Create unit tests for `AuthGuard` and `RoleGuard`
- [ ] 5.4 Create integration tests for full auth flow (register → login → me → refresh → logout)
- [ ] 5.5 Create integration tests for error cases (invalid credentials, expired token, insufficient role)

## 6. Validation & Testing

- [ ] 6.1 Generated project SHALL compile and pass `cargo test` with zero errors
- [ ] 6.2 Test with `ironic new temp-api && cd temp-api && ironic generate ready-resource auth && cargo test`
- [ ] 6.3 Test each variant individually: `auth-basic`, `auth-jwt`, `auth-oauth`
- [ ] 6.4 Verify auto-registration works (module appears in modules/mod.rs and app.rs)

## 7. Documentation

- [ ] 7.1 Create `docs/content/docs/getting-started/ready-resources.md` with usage guide
- [ ] 7.2 Add ready-resource to CLI reference docs
- [ ] 7.3 Add example to `examples/` demonstrating the generated auth module
