---
title: Fundamentals
description: Understand modules, dependency injection, routing, pipelines, and lifecycle.
---

# Fundamentals

## Modules

Every application starts from one root `Module`. Imports are traversed deterministically and
validated for cycles, duplicate ownership, private providers, missing dependencies, and invalid
exports.

## Dependency injection

Providers use concrete Rust types as tokens. Singleton and transient scopes, synchronous
constructors, asynchronous factories, values, optional dependencies, cycle detection, and local
test overrides are supported. Injectable fields use `Arc<T>` so sharing is explicit.

## Controllers and routes

Controllers own a path prefix. `#[routes]` turns HTTP method and parameter attributes into public
route metadata. JSON bodies, queries, path parameters, and headers are extracted before invoking
the handler.

## Request pipeline

Execution order is application middleware, controller middleware, route middleware, guards,
interceptors, extraction and validation, handler, then interceptor and middleware unwinding.
Request IDs and structured tracing spans are enabled on compiled applications.

## Lifecycle

Module initialization and application bootstrap run in dependency order. Application shutdown and
module destruction run in reverse order, including partial-startup cleanup.
