# RustFrame

RustFrame is an experimental modular, type-safe application framework for structured Rust APIs on top of Axum.

The project is being implemented from the contracts in [`rfcs/`](./rfcs) and the sequence in [`todo.md`](./todo.md). Version 0.1 is an experimental preview and is not yet recommended for production use.

Start with the [getting-started guide](./docs/content/docs/getting-started.md), study the [REST example](./examples/rest-api), and review the [known limitations](./RELEASE_NOTES.md) before adopting the preview.

## Workspace architecture

```text
rustframe facade
      │
rustframe-core ──▶ rustframe-di ◀── rustframe-config
      │            rustframe-common
      ▼
rustframe-http
      │
rustframe-platform
      │
rustframe-platform-axum ──▶ Axum / Tower / Tokio
rustframe-openapi ─────────▶ optional OpenAPI / Swagger integration
```

Dependency rules:

- `rustframe-common` owns shared identifiers and errors and has no framework-crate dependencies.
- `rustframe-di` owns provider registration and resolution and may depend only on `rustframe-common` plus general-purpose libraries.
- `rustframe-http` owns transport-neutral HTTP contracts and may depend on `common` and `di`.
- `rustframe-platform` owns adapter contracts and may depend on transport-neutral crates.
- `rustframe-core` orchestrates modules, DI, lifecycle, and compiled routes without depending on a concrete platform adapter.
- `rustframe-platform-axum` is the only crate allowed to expose Axum, Tower, or Hyper integration.
- `rustframe-macros` generates public API calls; it contains no runtime behavior.
- `rustframe-testing` builds on public core and platform contracts.
- `rustframe-cli` scaffolds and orchestrates projects without introducing runtime dependencies.
- `rustframe-openapi` discovers compiled routes and optionally wraps the Axum adapter with OpenAPI
  JSON and Swagger UI endpoints.
- `rustframe` is the user-facing facade and prelude.

These directions are enforced through Cargo manifests and reviewed by `cargo metadata` in CI.

## Request panic boundary

When Rust is built with panic unwinding, the Axum adapter catches panics at the outer request
boundary and returns a redacted `RF_HTTP_HANDLER_PANICKED` response. This isolates an individual
request but does not make panics safe or recoverable application control flow. With
`panic = "abort"`, process termination remains the Rust-defined behavior.

## Development

RustFrame uses Rust 1.85 or newer and Edition 2024.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo bench --workspace --no-run
cargo deny check
cargo audit
```
