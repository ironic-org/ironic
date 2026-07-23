---
title: Why Ironic
description: Why we built Ironic, and how it differs from other Rust web frameworks.
---

# Why Ironic

Ironic was built to fill a gap in the Rust web ecosystem — a full-featured application framework with the structure, safety, and developer experience that teams need for production APIs.

## The problem

Rust has excellent low-level HTTP libraries (hyper, tower, axum) and some micro-frameworks, but nothing that provides the **batteries-included application structure** that frameworks like NestJS, Spring Boot, or ASP.NET provide in their ecosystems. Teams building medium-to-large APIs in Rust face the same challenges repeatedly:

- **No standard project structure** — Every project invents its own module layout
- **No built-in dependency injection** — Dependencies are manually threaded through constructors
- **No module system** — No explicit wiring of components with dependency graphs
- **No lifecycle hooks** — No standard way to run code on startup/shutdown
- **No integrated auth/config/metrics** — You piece together libraries yourself

## What Ironic provides

Ironic combines the proven architectural patterns of frameworks like NestJS with Rust's unique strengths:

| Challenge | Ironic solution | Rust advantage |
|-----------|----------------|----------------|
| Project structure | CLI scaffolding with conventions | Consistent layout across teams |
| Dependency management | DI container with scoped providers | Type-safe, no runtime reflection |
| Module system | `Module` trait with dependency graph | Compile-time module validation |
| Configuration | Typed, layered config with hot-reload | Deserialization guarantees at compile time |
| Authentication | JWT, OAuth2, sessions | Full control over crypto, no JS interop |
| Error handling | `HttpError` with structured codes | `Result` types + `?` operator |
| Observability | Metrics, logging, tracing | Zero-cost abstractions, no GC pauses |
| Resilience | Retry, circuit breaker, bulkhead | Predictable latency, no runtime surprises |

## Design principles

1. **Batteries included** — Common needs (DI, config, auth, metrics, OpenAPI) are built-in
2. **Compile-time safety** — Module wiring, provider resolution, and route registration are verified at compile time
3. **Modular by default** — Everything is a module with explicit imports and exports
4. **Transport neutral** — Define your API once; expose over HTTP, WebSocket, GraphQL, or future protocols
5. **Production ready** — Structured logging, metrics, circuit breakers, rate limiting, and hot-reload are first-class features

## Who should use Ironic

Ironic is a good fit if:

- You prefer **structured, opinionated** frameworks over micro-frameworks
- Your project needs **dependency injection** and **modular architecture**
- You want **batteries included** — auth, config, metrics, OpenAPI out of the box
- You're coming from **NestJS, Spring Boot, or ASP.NET** and want similar patterns in Rust
- You're building a **production API** that will be maintained by a team over years

It may not be the right fit if:

- You need a **micro-framework** for a tiny single-file API
- You prefer **minimal dependencies** and want to compose your own stack
- You're building a **library** rather than an application
