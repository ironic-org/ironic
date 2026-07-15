# Tasks

## 1. CLI Wiring

- [x] 1.1 Add `ReadyResource` variant to `Generator` enum in `crates/ironic-cli/src/cli.rs` with `auth`, `auth-basic`, `auth-jwt`, `auth-oauth` sub-variants
- [x] 1.2 Add dispatch case in `crates/ironic-cli/src/commands/generate.rs` for `Generator::ReadyResource`
- [x] 1.3 Create `crates/ironic-cli/src/generators/ready_resource.rs` with `generate_ready_resource()` entry point

## 2. Auth Module Templates

- [x] 2.1 Create template for `User` entity (`id`, `email`, `password_hash`, `name`, `role`, `provider`, `created_at`)
- [x] 2.2 Create template for `Role` enum (`Admin`, `User`, `Moderator`)
- [x] 2.3 Create template for `PasswordService` with `hash()` and `verify()` using Argon2id
- [x] 2.4 Create template for `AuthService` with `register()`, `login()`, `refresh()`, `me()`, `logout()`, `oauth_login()`
- [x] 2.5 Create template for `AuthController` at `/auth` with all routes
- [x] 2.6 Create templates for DTOs: `RegisterDto`, `LoginDto`, `RefreshDto`, `TokenResponse`
- [x] 2.7 Create template for JWT token handling (issue, verify, refresh)
- [x] 2.8 Create template for OAuth2 flows (Google, GitHub) with PKCE
- [x] 2.9 Create template for session management (`InMemorySessionStore`)

## 3. Guards & Decorators

- [x] 3.1 Create template for `AuthGuard` — extracts JWT from Authorization header, sets current user in request context
- [x] 3.2 Create template for `RoleGuard` — checks user role against required roles
- [x] 3.3 Create template for `current_user` custom decorator using `create_param_decorator!`
- [x] 3.4 Create template for `roles` custom decorator

## 4. Module Wiring

- [x] 4.1 Create template for `AuthModule` (or variant-specific module) with providers, controllers, guards, exports
- [x] 4.2 Auto-register module in `src/modules/mod.rs` via `ensure_items()`
- [x] 4.3 Auto-import module in `src/app.rs` via `ensure_module_import()`

## 5. Test Templates

- [x] 5.1 Create unit tests for `PasswordService` (hash, verify, unique salts)
- [x] 5.2 Create unit tests for `AuthService` (register, login, refresh)
- [x] 5.3 Create unit tests for `AuthGuard` and `RoleGuard`
- [x] 5.4 Create integration tests for full auth flow (register → login → me → refresh → logout)
- [x] 5.5 Create integration tests for error cases (invalid credentials, expired token, insufficient role)

## 6. Validation & Testing

- [ ] 6.1 Generated project SHALL compile and pass `cargo test` with zero errors
- [ ] 6.2 Test with `ironic new temp-api && cd temp-api && ironic generate ready-resource auth && cargo test`
- [ ] 6.3 Test each variant individually: `auth-basic`, `auth-jwt`, `auth-oauth`
- [ ] 6.4 Verify auto-registration works (module appears in modules/mod.rs and app.rs)

## 7. Documentation

- [ ] 7.1 Create `docs/content/docs/getting-started/ready-resources.md` with usage guide
- [ ] 7.2 Add ready-resource to CLI reference docs
- [ ] 7.3 Add example to `examples/` demonstrating the generated auth module
