---
title: Ironic
description: A modular, type-safe Rust application framework built on Axum.
---

# Ironic documentation

Ironic provides explicit modules, dependency injection, controllers, request pipelines,
lifecycle hooks, testing utilities, and an Axum adapter without runtime reflection or global
mutable state.

## Start here

- [Getting started](/docs/getting-started) — build and run the first application.
- [Fundamentals](/docs/fundamentals) — modules, providers, controllers, and lifecycle.
- [Configuration](/docs/configuration) — typed sources, validation, and redacted secrets.
- [Security](/docs/security) — safe defaults and production integration guidance.
- [Examples](/docs/examples) — REST, validation, error handling, and testing.
- [Benchmarks](/docs/benchmarks) — reproducible framework overhead measurements.

The framework remains usable through explicit Rust APIs; procedural macros only generate calls to
those public contracts.
