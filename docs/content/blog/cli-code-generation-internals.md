---
title: "How ironic generate patches your source code — AST-level surgery"
description: "Deep dive into the CLI generator internals: how write_generated, ensure_items, and ensure_module_import perform safe, idempotent source code patching using syn and quote."
date: "2026-07-15"
author: "Ironic Team"
---

# How `ironic generate` patches your source code — AST-level surgery

Most CLI scaffolding tools are glorified string-replacers. `ironic generate` is different. It parses your existing source code into a full syntax tree, walks the AST, and surgically inserts new items — declarations, imports, module registrations — at exactly the correct position. It never overwrites user code, and running it twice never duplicates anything. Here's how it works.

## Architecture overview: generators produce strings, source.rs patches files

The generator pipeline lives in `crates/ironic-cli/src/generators/`. Every generator — `generate_resource`, `generate_controller`, `generate_module` — follows the same two-phase pattern:

**Phase 1 — Template rendering.** Each generator calls functions in `templates.rs` that take a `Names` struct (holding `snake`, `pascal`, and `kebab` case variants derived from the user's input) and return a `String` of valid Rust source code. For example, `templates::resource_module(&names)` produces a complete `#[derive(Module)]` struct with all the sub-module declarations and DI wiring pre-filled.

**Phase 2 — Filesystem patching.** The generated strings are handed to three core functions in `source.rs`: `write_generated`, `ensure_items`, and `ensure_module_import`. These are the surgeons. They don't just dump strings onto disk — they parse your project, analyze its structure, and make minimally invasive edits.

This separation matters. The templates know nothing about your filesystem. `source.rs` knows nothing about your domain model. Each layer is simple, testable, and — critically — safe to run repeatedly.

## `write_generated`: file creation with conflict detection

`write_generated` (`source.rs:8`) is the most straightforward of the three. It handles three states:

- **File doesn't exist:** Create parent directories, write the content, return `Ok(true)` (file was created).
- **File exists with identical content:** Return `Ok(false)` (nothing changed). This is the idempotency guarantee.
- **File exists with different content:** Return `Err(CliError::FileConflict { path })`. The CLI refuses to overwrite anything the user may have customized.

This is the fundamental safety invariant: `ironic generate` will **never** overwrite a file you've modified. It prefers to error and tell you exactly which file is in conflict so you can resolve it yourself.

The `write_module_shell` variant (`source.rs:27`) adds an extra check: if the file already contains a struct with the expected module name (e.g., `UsersModule`), it skips generation — the module shell already exists. Otherwise it fails with a conflict error.

## `ensure_items`: inserting declarations into existing source

This is where things get interesting. `ensure_items` (`source.rs:50`) reads an existing Rust source file, parses it with `syn`, and checks whether a set of declarations already exists. If not, it pushes them onto the `file.items` vector — inserting them at the end of the file — and writes back the formatted output via `prettyplease::unparse`.

The key insight is the deduplication strategy. Instead of naive string matching, `ensure_items` round-trips each candidate through `quote::quote!`:

```rust
let canonical = quote::quote!(#item).to_string();
let exists = file.items.iter().any(|existing| {
    quote::quote!(#existing).to_string() == canonical
});
```

By normalizing both the existing items and the new ones through `quote`, the comparison accounts for whitespace, formatting differences, and macro hygiene. If you've written `pub mod controller ;` with extra spaces, `quote` normalizes both sides to `pub mod controller;`, and the deduplication succeeds. This is what makes idempotency work — the canonical form is invariant across runs.

The function is used throughout the generator suite. `generate_controller` calls it on the module's `mod.rs` to insert `pub mod controller;` and `pub use controller::FooController;`. `generate_service` inserts `pub mod services;` and the corresponding `pub use`. `register_root_module` (`mod.rs:336`) inserts `pub mod users;` into `src/modules/mod.rs`. And `ensure_main_registration` (`mod.rs:347`) inserts `mod modules;` into `src/main.rs`.

## `ensure_module_import`: registering modules without breaking the `#[module]` attribute

This is the deepest surgery. `ensure_module_import` (`source.rs:128`) adds a type to the `imports` array of a `#[module(...)]` attribute — without touching any other part of the file.

Here's the problem it solves. Ironic modules declare their dependencies in a structured attribute:

```rust
#[module(imports = [WelcomeModule, ExampleModule], providers = [...], controllers = [...])]
pub struct AppModule;
```

To register a new module, you need to append to `imports = [...]` — but you also need to preserve all existing entries in `imports`, `providers`, `controllers`, and `exports`. You can't just do string insertion; you need to parse the attribute's internal syntax, modify the correct array, and regenerate the entire attribute.

The function does exactly that. It walks the file's items, finds the struct with a `#[module]` attribute, parses the attribute arguments into a `ModuleMetadata` struct (with its own `syn::parse::Parse` implementation at line 96), checks if the import already exists (using the same `quote` normalization trick), and if not, appends it and rewrites the attribute via `syn::parse_quote!`:

```rust
attribute.meta = syn::parse_quote!(module(
    imports = [#(#imports),*],
    providers = [#(#providers),*],
    controllers = [#(#controllers),*],
    exports = [#(#exports),*],
));
```

The entire file is then re-serialized through `prettyplease::unparse`, preserving all formatting. The caller, `ensure_app_import` (`mod.rs:363`), wires the new module's import path — e.g., `crate::modules::users::UsersModule` — into `src/app.rs`, or emits a manual instruction if the file doesn't exist or the attribute can't be parsed.

## A concrete trace: `ironic generate resource users`

Let's walk through what happens when you run `ironic generate resource users` in a fresh Ironic project.

**Step 1 — Name normalization.** The `Names::parse("users")` call produces `snake = "users"`, `pascal = "Users"`, `kebab = "users"`. These are used throughout every template.

**Step 2 — File generation.** `generate_resource` (`mod.rs:158`) defines a list of 13 file paths mapped to template outputs. Each is written via `write_generated`:

```
src/modules/users/mod.rs                    → Templates::resource_module
src/modules/users/controller/mod.rs         → Templates::controller_mod
src/modules/users/controller/users_controller.rs → Templates::resource_controller
src/modules/users/services/mod.rs           → Templates::services_mod
src/modules/users/services/users_service.rs → Templates::service
src/modules/users/dto/mod.rs                → Templates::dto_mod
src/modules/users/dto/create_users_dto.rs   → Templates::create_dto
src/modules/users/dto/update_users_dto.rs   → Templates::update_dto
src/modules/users/entities/mod.rs           → Templates::entities_mod
src/modules/users/entities/users.rs         → Templates::entity
src/modules/users/tests/mod.rs              → Templates::test_mod
src/modules/users/tests/unit.rs             → Templates::test_unit
src/modules/users/tests/integration.rs      → Templates::test_integration
```

**Step 3 — Module registration.** `register_root_module` calls `ensure_items` on `src/modules/mod.rs`, inserting `pub mod users;`. If the file was previously:

```rust
pub mod example;
```

It becomes:

```rust
pub mod example;
pub mod users;
```

**Step 4 — Root registration.** `ensure_main_registration` calls `ensure_items` on `src/main.rs` to insert `mod modules;` (if it isn't already there).

**Step 5 — App import.** `ensure_app_import` calls `ensure_module_import` on `src/app.rs`, adding `crate::modules::users::UsersModule` to the `imports = [...]` array. Before:

```rust
#[module(imports = [HealthModule, MetricsModule, WelcomeModule, ExampleModule])]
pub struct AppModule;
```

After:

```rust
#[module(imports = [HealthModule, MetricsModule, WelcomeModule, ExampleModule, crate::modules::users::UsersModule])]
pub struct AppModule;
```

All 13 files are created, the module registry is updated, and the app module is patched — in a single command invocation.

## What `generate resource` actually produces: a full CRUD vertical slice

The 13 generated files form a complete vertical slice:

- **Controller** (`users_controller.rs`): An `#[controller("/users")]` struct with routes for `GET /`, `GET /:id`, `POST /`, `PUT /:id`, `DELETE /:id`. Depends on `Arc<UsersService>` injected via the framework's DI container.
- **Service** (`users_service.rs`): An `#[derive(Injectable)]` struct. The default template returns a stub, but the module's `providers` array in `mod.rs` wires it into the DI graph — ready for you to add database calls.
- **DTOs** (`create_users_dto.rs`, `update_users_dto.rs`): Serializable request body types. `CreateUsersDto` includes a required `name: String` field; `UpdateUsersDto` makes it optional.
- **Entity** (`users.rs`): A `#[derive(Serialize, Deserialize)]` struct with `id: String` and `name: String`.
- **Tests** (`unit.rs`, `integration.rs`): Unit tests for the service in isolation, and integration tests that spin up a `TestApplication`, send real HTTP requests, and assert status codes and response bodies.

Everything is wired together in `mod.rs` via the `#[module]` attribute:

```rust
#[module(
    providers = [UsersService],
    controllers = [UsersController],
)]
pub struct UsersModule;
```

## Idempotency: running the generator twice

Every mutating operation in the pipeline is guarded by a deduplication check. When you run `ironic generate resource users` a second time:

- `write_generated` opens each of the 13 files, finds identical content, and returns `Ok(false)` — no writes, no errors.
- `ensure_items` on `src/modules/mod.rs` finds `pub mod users;` already present (via `quote` normalization), returns `Ok(false)`.
- `ensure_module_import` on `src/app.rs` finds `UsersModule` already in `imports`, returns `Ok(false)`.

The `GenerationReport` tracks which files were created vs. unchanged, so you get clean output: "13 files unchanged." Nothing is duplicated. No `users2_service.rs` appears. No `imports` array grows unboundedly.

## The `ready-resource` generator: composing complex modules

Beyond CRUD scaffolding, the CLI ships "ready resource" generators that produce entire complex modules from templates. The `ready_resource` module (`ready_resource.rs`) provides four auth variants:

| Command | What it generates |
|---|---|
| `ironic generate ready-resource auth myapp` | Full auth: passwords, JWT, OAuth, sessions, RBAC |
| `ironic generate ready-resource basic` | Passwords + sessions only |
| `ironic generate ready-resource jwt` | JWT-only authentication |
| `ironic generate ready-resource oauth` | OAuth-only authentication |

Similarly, `file_upload_email.rs` provides `ironic generate ready-resource email` (SMTP/SES/SendGrid/Mailgun backends) and `ironic generate ready-resource file-upload` (local/S3/R2/Azure/GCS storage backends).

Each ready-resource generator follows the same `source.rs` pipeline: it defines a list of `(PathBuf, String)` files computed from templates, writes them through `write_generated`, registers the module through `ensure_items`, and adds the import through `ensure_module_import`. The difference is scale — a full auth module can span a dozen files with hundreds of lines of security-critical code — but the patching machinery is identical.

## Why AST-level surgery matters

The alternative — regex-based insertion, comment markers like `// IRONIC: GENERATED`, or template engines that replace entire files — all have failure modes. Comments get moved or deleted. Regex misses edge cases in macro bodies. Template engines force you to choose between hand-editing your module file and regenerating it from scratch.

By operating on the AST, `ironic generate` achieves three things simultaneously: it never clobbers user edits, it never duplicates code, and it works on any valid Rust source file regardless of formatting style. The `syn` + `quote` + `prettyplease` stack makes this possible in under 400 lines of code. That's the power of treating source code as structured data, not text.
