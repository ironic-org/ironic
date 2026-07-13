# Dependency Management

Ironic uses a centralized workspace dependency model. All dependency versions are defined once in `Cargo.toml` under `[workspace.dependencies]` and referenced by individual crates via `<name>.workspace = true`.

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

[Dependabot](https://docs.github.com/en/code-security/dependabot) is configured in `.github/dependabot.yml` and opens weekly PRs for version bumps. CI (`cargo test`, `cargo clippy`, `cargo audit`) must pass before merging.

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
