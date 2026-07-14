## Why

The current `ironic generate resource` command produces a basic CRUD scaffold (controller, service, DTOs, entities), but developers must manually wire up authentication, authorization, guards, role-based access, password hashing, JWT tokens, social login, and session management for every new project. This is repetitive, error-prone, and delays time-to-production. A `ready-resource` generator that produces a complete, production-grade auth module instantly would eliminate weeks of boilerplate and give every new Ironic project secure authentication out of the box.

## What Changes

- New CLI subcommand: `ironic generate ready-resource <name>` that generates a fully working authentication/authorization module
- The generated module includes: password hashing (Argon2id), JWT token issuance and verification, OAuth2 social login (Google, GitHub), session management, role-based access control (RBAC), and route guards
- Auto-generates: user entity, auth service, auth controller (register, login, logout, me, refresh), social login callback, role enum, permission decorator, role guard, and full test suite
- Multiple ready-resource variants: `auth` (full auth), `auth-basic` (passwords only), `auth-jwt` (JWT only), `auth-oauth` (OAuth only)
- `[- ready-resource-generator]` modifies the existing `cli-tooling` spec to add new generator subcommands

## Capabilities

### New Capabilities
- `ready-resource-auth`: Generates a complete authentication module with password hashing, JWT tokens, OAuth2 social login, sessions, role-based access control, and guards. Includes User entity, AuthService, AuthController (register/login/logout/me/refresh/callback), role enum, permission decorator, role guard, and full test suite.

### Modified Capabilities
- `cli-tooling`: Add `ironic generate ready-resource <variant>` subcommand with `auth`, `auth-basic`, `auth-jwt`, `auth-oauth` variants. The generator must produce a working module immediately — no manual wiring required.

## Impact

- New crate: `crates/ironic-cli/src/generators/ready_resource.rs` (generator logic + templates)
- Modified: `crates/ironic-cli/src/cli.rs` (new subcommand), `crates/ironic-cli/src/commands/generate.rs` (dispatch)
- New templates: auth service, auth controller, user entity, role enum, guards, DTOs, tests
- Dependencies added for generated projects: `jsonwebtoken`, `argon2`, `oauth2`
- Docs: new page `docs/content/docs/getting-started/ready-resources.md`
