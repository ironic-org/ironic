---
title: Ironic
description: The complete beginner's guide to building APIs with Ironic — a Rust framework that makes backend development simple and fun.
---

# Welcome to Ironic

Ironic is a **batteries-included, type-safe application framework for Rust**. Think of it like LEGO bricks for your backend — each piece snaps together cleanly, and the compiler tells you if something's wrong before you even run the code.

> **No prior framework experience needed.** If you know basic Rust (structs, functions, `async`), you can build a production API by the end of this guide.

## Why a framework?

Building a production API involves solving the same problems every time: routing, configuration, dependency management, authentication, error handling, validation, serialization, observability, and more. A framework codifies these solutions so you don't have to reinvent them for every project.

Ironic is designed for teams and projects where:

- **Consistency matters** — A predictable project structure makes onboarding faster
- **Correctness is critical** — The type system catches wiring mistakes at compile time
- **You need more than a router** — Real-world apps need DI, middleware, background jobs, auth, and metrics out of the box
- **You want longevity** — Modular architecture means you can swap implementations without rewriting your app

## Philosophy

Ironic follows these principles:

| Principle | What it means |
|-----------|---------------|
| **Batteries included** | Common needs (DI, config, auth, metrics, OpenAPI) are built-in, not bolted on |
| **Compile-time safety** | Module wiring, provider resolution, and route registration are verified at compile time |
| **Modular by default** | Everything is a module with explicit imports and exports — no hidden global state |
| **Transport neutral** | Define your API once; expose it over HTTP, WebSocket, GraphQL, or any future protocol |
| **Production ready** | Structured logging, metrics, circuit breakers, rate limiting, and hot-reload are first-class features |

## Feature overview

| Area | What Ironic provides |
|------|---------------------|
| **Routing** | Controllers with path parameters, query strings, body extraction, versioning |
| **DI Container** | Singleton, transient, and request-scoped providers with cycle detection |
| **Configuration** | Layered files, environment variables, profiles, hot-reload, secret redaction |
| **Authentication** | JWT, OAuth2, session-based auth with guards and middleware |
| **Data Access** | SQLx, SeaORM, Diesel, MongoDB, Redis — first-class integrations |
| **Security** | CORS, CSRF, rate limiting, security headers |
| **Observability** | Prometheus metrics, structured JSON logging, OpenTelemetry tracing |
| **Resilience** | Retry with backoff, circuit breaker, bulkhead/concurrency limit |
| **API Docs** | Automatic OpenAPI/Swagger generation from route definitions |
| **Real-time** | WebSocket gateways with rooms, broadcasting, SSE channels |
| **Background Work** | Cron scheduling, event bus, queues, sagas, CQRS |
| **CLI Tooling** | Code generation, project scaffolding, debug REPL, migration management |
| **Testing** | Test module builder, in-process HTTP client, fluent assertions |

## What you'll learn

This documentation walks you through every feature step by step:

| Section | What you'll build |
|---------|-------------------|
| [Getting Started](/docs/getting-started/getting-started) | Install the CLI, create your first project, and see it running in 60 seconds |
| [Project Structure](/docs/project-structure/overview) | Every file and folder explained — how modules, controllers, services, and repositories connect |
| [Core Concepts](/docs/fundamentals/overview) | Understand Modules, Controllers, Services, and Dependency Injection — the 4 building blocks |
| [CLI Reference](/docs/getting-started/cli) | Master the command-line tools for scaffolding, generating code, and inspecting your app |
| [Configuration](/docs/configuration/overview) | Load settings from files, environment variables, and keep secrets safe |
| [HTTP & API](/docs/http-api/api-versioning) | Routes, versioning, validation, error handling, serialization, compression, and OpenAPI |
| [Security](/docs/http-api/security) | CORS, rate limiting, CSRF protection, and security headers |
| [Database & Auth](/docs/data-auth/database-integrations) | Connect to PostgreSQL, MySQL, MongoDB, Redis. Add login with JWT, OAuth, or sessions |
| [Performance](/docs/performance/cache-decorators) | Caching, background jobs, and cron scheduling |
| [Distributed Systems](/docs/distributed/overview) | Microservices, queues, sagas, events, and the transactional outbox |
| [Advanced](/docs/advanced/sessions) | Sessions, multipart uploads, static files, and devtools plugins |
| [Observability](/docs/observability/overview) | Metrics, tracing, and production monitoring |

## How Ironic compares

| | Ironic | Axum | Actix-Web | Rocket | Salvo | Poem | Warp | NestJS |
|---|---|---|---|---|---|---|---|---|
| **DI Container** | ✅ Built-in | ❌ Bring your own | ❌ Bring your own | ❌ | ❌ | ❌ | ❌ | ✅ Built-in |
| **Module System** | ✅ Built-in | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Built-in |
| **CLI** | ✅ Scaffolding + generators | ❌ | ❌ | 🔶 Basic | ❌ | ❌ | ❌ | ✅ CLI |
| **Auth** | ✅ JWT + OAuth2 + Sessions | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Passport |
| **OpenAPI** | ✅ Auto-generated | ❌ | 🔶 Utopică | ❌ | ❌ | ✅ Poem OpenAPI | ❌ | ✅ Swagger |
| **Metrics** | ✅ Prometheus | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Config** | ✅ Typed + hot-reload | ❌ | ❌ | 🔶 Figment | 🔶 | 🔶 | ❌ | ✅ ConfigModule |
| **WebSockets** | ✅ Gateways + Rooms | ✅ axum/ws | ✅ actix-ws | ❌ | ✅ | ✅ | ❌ | ✅ Gateways |
| **GraphQL** | ✅ async-graphql | ❌ | 🔶 | ❌ | ❌ | ✅ | ❌ | ✅ @nestjs/graphql |
| **Background Jobs** | ✅ Cron + Queues + Sagas | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ @nestjs/schedule |
| **Caching** | ✅ In-memory + Redis | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ CacheModule |
| **CQRS / Event Bus** | ✅ Built-in | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ @nestjs/cqrs |
| **Validation** | ✅ Pipes + Garde | ❌ | ❌ | ❌ | ✅ Validator | ✅ Validator | ❌ | ✅ ValidationPipe |
| **Testing Utilities** | ✅ TestModule + in-process client | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Test |
| **Hot Reload** | ✅ Config + file watching | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Webpack HMR |
| **Middleware** | ✅ Guards + Interceptors + Filters | ✅ Tower layers | ✅ Middleware | ✅ Fairings | ✅ Middleware | ✅ Middleware | ✅ Filters | ✅ Guards + Interceptors |
| **Runtime** | Async (tokio) | Async (tokio) | Async (tokio) | Async (tokio) | Async (tokio) | Async (tokio) | Async (tokio) | Single-threaded (Node.js) |
| **Memory Safety** | ✅ Compile-time | ✅ Compile-time | ✅ Compile-time | ✅ Compile-time | ✅ Compile-time | ✅ Compile-time | ✅ Compile-time | ❌ GC |
| **Package ecosystem** | Single crate | Tower/tower-http | actix extras | Rocket contrib | Salvo extras | Poem extras | Filters | NPM (1M+ packages) |

Ironic gives you all of this **out of the box**, so you can focus on what makes your application unique. No other Rust framework matches this breadth of built-in features — and unlike NestJS, you get Rust's compile-time safety and native performance.

## Where to start

- New to Ironic? Start with [Getting Started](/docs/getting-started/getting-started)
- Coming from NestJS? Read [Coming from NestJS](/docs/getting-started/coming-from-nestjs)
- Want to see benchmarks? Check [Benchmarks](/docs/more/benchmarks)
- Ready to build? Install the [CLI](/docs/getting-started/cli) and scaffold a project
