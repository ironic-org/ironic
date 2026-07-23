---
title: Introduction
description: What is Ironic? A batteries-included, type-safe application framework for Rust.
---

# Introduction

Ironic is a **batteries-included, type-safe application framework for Rust**. It gives you a structured, opinionated way to build web APIs and backend services — with dependency injection, modular architecture, and deep compile-time guarantees.

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

## Where to start

- New to Ironic? Start with [Getting Started](/docs/getting-started/getting-started)
- Coming from NestJS? Read [Coming from NestJS](/docs/getting-started/coming-from-nestjs)
- Want to see benchmarks? Check [Benchmarks](/docs/getting-started/benchmarks)
- Ready to build? Install the [CLI](/docs/getting-started/cli) and scaffold a project
