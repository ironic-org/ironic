# Ironic

[![CI](https://github.com/ironic-org/ironic/actions/workflows/ci.yml/badge.svg)](https://github.com/ironic-org/ironic/actions/workflows/ci.yml)
[![Release](https://github.com/ironic-org/ironic/actions/workflows/release.yml/badge.svg)](https://github.com/ironic-org/ironic/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/ironic.svg)](https://crates.io/crates/ironic)
[![Docs.rs](https://img.shields.io/docsrs/ironic)](https://docs.rs/ironic)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.97+-orange)](rust-toolchain.toml)
[![Discord](https://img.shields.io/discord/ironic?label=discord&logo=discord)](https://discord.gg/ironic-community)
[![Sponsor](https://img.shields.io/badge/sponsor-30363D?logo=GitHub-Sponsors&logoColor=#EA54AE)](https://github.com/sponsors/morshedulmunna)

A batteries-included, type-safe application framework for building structured Rust APIs on top of Axum. Inspired by NestJS's modular architecture, grounded in Rust's type system.

## Quick Start

```bash
cargo install ironic
ironic new my-api
cd my-api
ironic start
```

## Features

- **Modular architecture** — modules, imports, exports, provider visibility
- **Dependency injection** — singletons, transients, factories, cycle detection
- **HTTP routing** — Axum adapter, controllers, parameter extraction
- **API versioning** — URI prefix, header-based, and media-type versioning strategies
- **Request pipeline** — middleware, guards, interceptors, error handling
- **Parameter pipes** — type parsing, validation, and transformation pipelines
- **Validation pipes** — `ValidationPipe` with `garde` integration (`#[garde(...)]` attributes)
- **Exception filters** — structured error handling with route-level and global filter chains
- **Response serialization** — `#[derive(Serializable)]` with `#[exclude]` and `#[expose(role)]` for field-level JSON control
- **Security middleware** — CORS, rate limiting, security headers (HSTS, CSP, X-Content-Type-Options, X-Frame-Options), CSRF protection
- **Response compression** — gzip, brotli, and zstd via `AxumAdapter::compression()`
- **Procedural macros** — `#[derive(Injectable)]`, `#[Module]`, `#[controller]`, `#[get]`, `#[post]`, `#[derive(Serializable)]`
- **Testing utilities** — in-process test app, provider overrides, fluent assertions
- **CLI** — project scaffolding, code generators, doctor command
- **OpenAPI** — automatic schema generation, Swagger UI
- **Integrations** — SQLx, SeaORM, Diesel, MongoDB, Redis, JWT, OAuth, gRPC, GraphQL

## Example

```rust
use ironic::prelude::*;

#[derive(Injectable)]
#[controller("/users")]
struct UsersController {
    service: Arc<UsersService>,
}

#[routes]
impl UsersController {
    #[get("/:id")]
    async fn find_one(&self, #[param] id: Uuid) -> Json<UserResponse> {
        Json(self.service.find_one(id).await)
    }
}

#[derive(Module)]
#[module(controllers = [UsersController], providers = [UsersService])]
struct UsersModule;

#[ironic::main]
async fn main() {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build()
        .await
        .unwrap()
        .listen("127.0.0.1:3000")
        .await
        .unwrap();
}
```

## Documentation

- [Getting Started](./docs/content/docs/getting-started.md)
- [Full Documentation](https://docs.rs/ironic)
- [Examples](./examples/)
- [Release Notes](./RELEASE_NOTES.md)

## Contributing

We welcome contributions! Here's how to get started.

### First Time

1. Find an issue tagged [good first issue](https://github.com/ironic-org/ironic/labels/good%20first%20issue) or [help wanted](https://github.com/ironic-org/ironic/labels/help%20wanted)
2. Comment on the issue that you're working on it
3. Follow the steps below to open a PR

### Setup

```bash
git clone https://github.com/ironic-org/ironic.git
cd ironic
cargo build
cargo test
```

### Branch Naming

```
feat/description     # new features
fix/description      # bug fixes
chore/description    # tooling, CI, deps
docs/description     # documentation
refactor/description # code restructuring
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add #[ironic::test] proc-macro
fix: resolve DI cycle detection panic
chore: bump axum to 0.8.10
docs: update release workflow guide
```

### PR Workflow

1. Create a branch from `main`: `git checkout -b feat/my-feature`
2. Make your changes
3. Run checks locally:
   ```bash
   cargo build
   cargo test --all-features
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   ```
4. Commit and push: `git push origin feat/my-feature`
5. Open a PR against `main` with:
   - Clear title matching commit conventions
   - Description explaining what and why
   - `Closes #N` to link the issue
6. Ensure CI passes on your PR
7. Address reviewer feedback with additional commits

### CI Pipeline

| Check | Description |
|-------|-------------|
| Formatting | `cargo fmt --check` |
| Clippy | `cargo clippy -D warnings` (all features) |
| Test | `cargo test --all-features` (stable + nightly) |
| Audit | `cargo audit` (vulnerability scan) |
| Deny | `cargo deny` (license + duplicate check) |
| Docs | Build docs site |
| Fuzz | 60s smoke test on nightly |

### Code Style (Enforced — PRs Must Comply)

PRs that violate these rules **will not be merged**:

- **Patterns** — follow existing conventions in the codebase
- **No comments** — code should be self-documenting. Comments only for non-obvious logic
- **Small functions** — each function should have a single responsibility
- **Document all public APIs** — every public type, method, and module export needs a doc comment
- **Tests required** — new features must include tests; bug fixes must include a regression test

All items are checked via the PR template checklist. CI enforces what it can (fmt, clippy, tests); the rest is enforced during review.

### Need Help?

- [Discord](https://discord.gg/ironic-community)
- [Discussions](https://github.com/ironic-org/ironic/discussions)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## Support

If Ironic helps you build something awesome, consider [sponsoring](https://github.com/sponsors/morshedulmunna) the project. Your support covers hosting, tooling, and development time.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.
