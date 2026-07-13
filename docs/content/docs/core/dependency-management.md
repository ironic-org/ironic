---
title: Dependency management
description: Workspace dependency versioning, optional dependencies, and feature flag configuration.
---

# Dependency management

Ironic uses a centralized workspace dependency model. All dependency versions are defined once in
`Cargo.toml` under `[workspace.dependencies]` and referenced by individual crates via
`<name>.workspace = true`.

## Feature flags

Enable only the capabilities your project needs:

```toml
ironic = { features = [
    "security",           # CORS, rate limiting, headers, CSRF
    "validation",         # ParseIntPipe, ValidationPipe, etc.
    "versioning",         # API versioning
    "serialization",      # Field-level serialization rules
    "compression",        # gzip, brotli, zstd
    "cache",              # In-memory and Redis caching
    "scheduling",         # Cron and interval scheduling
    "realtime",           # WebSocket gateways
    "cron",               # Cron expression parsing
    "database",           # SQLx, SeaORM, Diesel, MongoDB, Redis
    "authentication",     # Argon2, JWT, OAuth2, sessions
    "distributed",        # Queues, microservices, CQRS, sagas, gRPC, GraphQL
] }
```

## Optional DI dependencies

Mark injectable fields as optional with the `#[injectable(optional = [...])]` attribute:

```rust
use ironic::Injectable;

#[injectable(optional = [Logger])]
struct ReportingService {
    database: Arc<DatabaseConnection>,
    logger: Option<Arc<Logger>>, // optional — resolves to None when Logger is not registered
}
```

```rust
use ironic::{Injectable, Dependency, Dependency::required, Dependency::optional};

#[injectable(optional = [Notifier])]
struct OrderService {
    notifier: Option<Arc<Notifier>>, // None in testing, Some in production
}
```

The framework generates `Dependency::optional` for listed types and skips validation when the
provider is not registered. The field type must be `Option<Arc<T>>`.

## Version Strategy

- Pin minor versions (e.g. `"0.8"`) for core framework deps — deliberate upgrades only
- Patch versions auto-resolve via `Cargo.lock`
- Lockfile is committed to ensure reproducible CI builds

## Keeping Dependencies Updated

```bash
# Check what's outdated
cargo install cargo-edit
cargo outdated --workspace

# Upgrade everything to latest compatible versions
cargo upgrade --workspace

# Check for security advisories
cargo audit
```

## Automated Updates

[Dependabot](https://docs.github.com/en/code-security/dependabot) is configured in
`.github/dependabot.yml` and opens weekly PRs for version bumps. CI (`cargo test`,
`cargo clippy`, `cargo audit`) must pass before merging.

## Breaking Changes

When upgrading a dep with breaking changes:

1. Check the dep's changelog / migration guide
2. Update usages across all crates in the workspace
3. Run `cargo test --workspace --all-features`
4. Update the minimum pinned version if needed
5. Review `RELEASE_NOTES.md` for public API consumers

## Vendoring (Offline / Air-Gapped)

```bash
cargo vendor vendor
```

Add this to `.cargo/config.toml` when working offline:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
```
