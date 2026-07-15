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

Ironic combines NestJS's batteries-included philosophy with Rust's performance and zero-cost abstractions:

| Feature | NestJS | Axum | Actix Web | **Ironic** |
|---------|--------|------|-----------|------------|
| **Language** | TypeScript | Rust | Rust | **Rust** |
| **Architecture** | Decorator modules | Handler functions | Actor system | **Module graph + DI** |
| **Dependency Injection** | ✅ Built-in | — Third-party | — Third-party | **✅ Built-in** |
| **Scope-aware DI** | ✅ | ❌ | ❌ | **✅** |
| **Middleware pipeline** | ✅ Nest middleware | Tower layers | Middleware wrap | **✅ + Guards + Interceptors** |
| **CLI scaffolding** | ✅ | ❌ | ❌ | **✅ `ironic generate`** |
| **Rate limiting built-in** | ThrottlerModule | ❌ | ❌ | **✅** |
| **Security headers built-in** | Helmet | ❌ | ❌ | **✅** |
| **Cron / scheduled tasks** | ✅ | ❌ | ❌ | **✅** |
| **OpenAPI generation** | ✅ | Utoipa | Utoipa | **✅** |
| **WebSockets built-in** | ✅ | ✅ | ✅ | **✅** |
| **Feature flags** | ❌ | ❌ | ❌ | **✅ Compile-time** |
| **Learning curve** | Moderate | Low | Medium | **Moderate** |
| **Ecosystem maturity** | Mature (2017) | Growing (2021) | Mature (2017) | **Early (2025)** |

> ✅ = built-in  ·  — = needs third-party crate  ·  ❌ = not available

## How the docs work

Every page follows the same structure:

1. **What you'll learn** — goals for the section
2. **The big picture** — a simple analogy or diagram
3. **Step-by-step code** — copy-paste examples with line-by-line explanations
4. **Try it yourself** — a quick exercise
5. **Common mistakes** — things that trip people up
6. **What you learned** — a summary checklist

## Quick navigation

### I'm new here
Start with [Getting Started](./getting-started) → [Fundamentals](./fundamentals) → [CLI](./cli)

### I want to build an API
[Fundamentals](./fundamentals) → [API Versioning](./api-versioning) → [Validation](./validation-pipes) → [Error Handling](./exception-filters)

### I want to add a database
[Database Integrations](./database-integrations) → [Authentication](./authentication)

### I'm deploying to production
[Security](./security) → [Observability](./observability) → [Performance](./cache-decorators)

---

**Ready?** Start with [Getting Started](./getting-started) — you'll have a running API in under a minute.
