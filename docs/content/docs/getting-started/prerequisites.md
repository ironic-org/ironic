---
title: Prerequisites
description: What you need to know before using Ironic — Rust fundamentals, async concepts, and tooling.
---

# Prerequisites

Ironic is a Rust framework, so you need basic Rust knowledge to use it effectively. Here's what you should know before diving in.

## Required knowledge

### Rust fundamentals

You should be comfortable with:

| Concept | Why it matters for Ironic |
|---------|--------------------------|
| **Structs and enums** | Controllers, services, and configs are all structs |
| **Functions and methods** | Route handlers, guards, and pipes are functions |
| **`impl` blocks** | Module definitions, route implementations, service logic |
| **`Result<T, E>`** | Every fallible operation returns `Result` |
| **`Option<T>`** | Optional parameters, nullable fields, conditional values |
| **`Arc<T>`** | Shared ownership for DI-injected dependencies |
| **Traits** | `Module`, `Guard`, `Interceptor`, `ExceptionFilter` are all traits |
| **Async/await** | Route handlers, DI resolution, and middleware are async |
| **Derive macros** | `#[derive(Debug, Clone, Serialize)]` is used everywhere |
| **Generics** | `JsonBody<T>`, `Arc<T>`, `Provider<T>` use generics |
| **`match` and pattern matching** | Error handling, state machines, option/result handling |

### Tooling

| Tool | Required? | Purpose |
|------|-----------|---------|
| Rust (rustc + cargo) | ✅ Required | Compiler and build tool |
| Cargo | ✅ Required | Dependencies, builds, tests |
| `cargo-generate` | ✅ Recommended | Used by `ironic new` for scaffolding |
| Docker | 🔶 Optional | Local databases (PostgreSQL, Redis) |
| `trunk` / wasm-pack | ❌ Not needed | Only for WASM targets |

## Nice-to-have

These aren't required but will make your life easier:

- **Familiarity with NestJS or ASP.NET** — Ironic's architecture mirrors these frameworks
- **SQL knowledge** — If using SQLx or SeaORM for database access
- **Docker** — For running local development databases
- **Basic understanding of HTTP** — Status codes, headers, methods, content types

## Learning resources

If you're new to Rust, work through these before starting with Ironic:

1. **[The Rust Book](https://doc.rust-lang.org/book/)** — Chapters 1-10 cover everything you need
2. **[Rust by Example](https://doc.rust-lang.org/rust-by-example/)** — Quick, practical examples
3. **[Tour of Rust](https://tourofrust.com/)** — Interactive learning
4. **[Async Book](https://rust-lang.github.io/async-book/)** — For understanding async Rust

### Quick checklist

Before starting your first Ironic project, make sure you can:

- [ ] Write a struct with methods
- [ ] Use `Result<T, E>` and the `?` operator
- [ ] Understand `Option<T>` and common combinators (`map`, `unwrap_or`, `ok_or`)
- [ ] Use `match` and `if let` patterns
- [ ] Define and implement a trait
- [ ] Write async functions and await futures
- [ ] Use `Arc<T>` for shared ownership
- [ ] Read and write Cargo.toml dependencies
- [ ] Run `cargo build`, `cargo test`, `cargo check`

## What you DON'T need

Ironic handles the complexity so you don't have to:

- ❌ **No manual HTTP server setup** — The platform adapter handles it
- ❌ **No manual dependency wiring** — The DI container resolves everything
- ❌ **No manual OpenAPI generation** — It's automatic from routes
- ❌ **No manual error type boilerplate** — `HttpError` covers common cases
- ❌ **No manual thread management** — Tokio runtime is configured automatically
