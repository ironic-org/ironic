# Ironic

Ironic is an experimental, batteries-included application framework for structured Rust APIs on
top of Axum. Applications add one dependency and use one CLI; runtime, DI, HTTP, Axum, OpenAPI,
testing utilities, configuration, and project generation are all exposed by the `ironic` crate.

Version 0.1 is a preview and is not yet recommended for production use.

## Install the CLI

The CLI ships as the `ironic` binary in the main crate:

```bash
cargo install ironic
ironic new my-api
cd my-api
ironic start
```

Generate a complete resource with:

```bash
ironic generate resource products
```

Generated applications contain one framework dependency:

```toml
[dependencies]
ironic = "0.1"
```

## Minimal application

```rust,no_run
use ironic::{AxumAdapter, prelude::*};

#[derive(Module)]
#[module()]
struct AppModule;

#[ironic::main]
async fn main() {
    FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build()
        .await
        .expect("application must initialize")
        .listen("127.0.0.1:3000")
        .await
        .expect("application server failed");
}
```

See the [getting-started guide](./docs/content/docs/getting-started.md), the
[REST example](./examples/rest-api), and the [release notes](./RELEASE_NOTES.md).

## Publishing

Rust requires procedural macros to live in a proc-macro crate. Therefore `ironic-macros` is a
small implementation companion, while `ironic` remains the only crate users install or declare.
Publish the companion first, then the public crate:

```bash
cargo login
cargo publish -p ironic-macros
cargo publish -p ironic
```

Before publishing, verify exactly what crates.io will receive:

```bash
cargo package -p ironic-macros
cargo package -p ironic
```

## Development

Ironic uses Rust 1.85 or newer and Edition 2024.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```
