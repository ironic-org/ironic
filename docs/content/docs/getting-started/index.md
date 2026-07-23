---
title: Ironic
description: The complete beginner's guide to building APIs with Ironic — a Rust framework that makes backend development simple and fun.
---

# Welcome to Ironic

Ironic is a **Rust framework for building web APIs**. Think of it like LEGO bricks for your backend — each piece snaps together cleanly, and the compiler tells you if something's wrong before you even run the code.

> **No prior framework experience needed.** If you know basic Rust (structs, functions, `async`), you can build a production API by the end of this guide.

## What you'll learn

This documentation walks you through every feature step by step:

| Section | What you'll build |
|---------|-------------------|
| [Getting Started](./getting-started) | Install the CLI, create your first project, and see it running in 60 seconds |
| [Project Structure](./project-structure) | Every file and folder explained — how modules, controllers, services, and repositories connect |
| [Core Concepts](./fundamentals) | Understand Modules, Controllers, Services, and Dependency Injection — the 4 building blocks |
| [CLI Reference](./cli) | Master the command-line tools for scaffolding, generating code, and inspecting your app |
| [Configuration](./configuration) | Load settings from files, environment variables, and keep secrets safe |
| [HTTP & API](./api-versioning) | Routes, versioning, validation, error handling, serialization, compression, and OpenAPI |
| [Security](./security) | CORS, rate limiting, CSRF protection, and security headers |
| [Database & Auth](./database-integrations) | Connect to PostgreSQL, MySQL, MongoDB, Redis. Add login with JWT, OAuth, or sessions |
| [Performance](./cache-decorators) | Caching, background jobs, cron scheduling, and distributed systems |
| [Advanced](./websocket-gateways) | WebSockets, custom decorators, plugins, and devtools |
| [Observability](./observability) | Metrics, tracing, and production monitoring |

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
