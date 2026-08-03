---
title: Quick Learn
description: Fast-track reference — modules, macros, architecture, features, and patterns all in one place.
---

# Quick Learn

The Quick Learn section is a fast-track reference for the `ironic` crate. Instead of
jumping between many sections, these pages give you a dense, code-first overview of
everything the framework offers.

> **Why this matters:** if you already know what you want to build, these pages get you
> to working code fast. When you need depth, every page links out to the dedicated
> section.

## What you'll learn

- The public API surface: modules, re-exports, core types, and macros
- How the framework fits together: lifecycle, DI, modules, routing, middleware
- Feature-by-feature usage: security, observability, databases, transports
- Battle-tested patterns: layering, error handling, configuration, testing

## The reference pages

| Page | Covers |
|------|--------|
| [API Reference](/docs/quick-learn/api-reference) | Public modules, re-exported crates, core types & traits, all macros, error codes, minimum Cargo.toml |
| [Architecture](/docs/quick-learn/architecture) | Application lifecycle, dependency injection, module system, routing, middleware, container internals |
| [Features](/docs/quick-learn/features) | Security, observability, database & storage, transports |
| [Patterns](/docs/quick-learn/patterns) | Common patterns, configuration, testing, best practices, troubleshooting |

## Feature flags

Every Cargo feature — grouped by category (Database, Authentication, Distributed
Systems, Transport, Security, Observability, and more) — is documented in the
[Feature Flag Reference](/docs/more/feature-flags).

## Where to go next

- New to Ironic? Start at [Getting Started](/docs/getting-started/getting-started)
- Building an API? See [Validation & pipes](/docs/http-api/validation-pipes)
- Scaling out? See [Distributed Systems](/docs/distributed/overview)
