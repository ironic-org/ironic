# Ironic

[![CI](https://github.com/ironic-org/ironic/actions/workflows/ci.yml/badge.svg)](https://github.com/ironic-org/ironic/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ironic.svg)](https://crates.io/crates/ironic)
[![Docs.rs](https://img.shields.io/docsrs/ironic)](https://docs.rs/ironic)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange)](rust-toolchain.toml)
[![Discord](https://img.shields.io/discord/ironic?label=discord&logo=discord)](https://discord.gg/ironic)

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
- **HTTP routing** — Axum adapter, controllers, parameter extraction, validation
- **Request pipeline** — middleware, guards, interceptors, error handling
- **Procedural macros** — `#[derive(Injectable)]`, `#[Module]`, `#[controller]`, `#[get]`, `#[post]`
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

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions, coding conventions, and PR workflow.

- [Good First Issues](https://github.com/ironic-org/ironic/labels/good%20first%20issue)
- [Help Wanted](https://github.com/ironic-org/ironic/labels/help%20wanted)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.
