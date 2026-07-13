## Why

Rust backend developers lack a structured, opinionated application framework that provides modular architecture, dependency injection, and a consistent request lifecycle while remaining faithful to Rust's type system. Existing patterns are ad-hoc, making large applications hard to organize, test, and extend. This change delivers the initial Ironic framework — a modular, type-safe Rust web framework built on Axum, inspired by NestJS's architectural consistency but grounded in Rust's ownership model and compile-time guarantees.

## What Changes

- Implement a modular application kernel with module imports, exports, providers, and controllers
- Implement a dependency injection container supporting singleton and transient scopes with cycle detection
- Implement a module compiler that validates provider graphs, detects cycles, and computes initialization order
- Implement transport-neutral HTTP contracts (methods, statuses, requests, responses, headers)
- Implement an Axum platform adapter that converts framework routes into an Axum router
- Implement a deterministic request pipeline: middleware → guards → interceptors → parameter extraction → validation → handler → response
- Implement lifecycle hooks (onModuleInit, onApplicationBootstrap, onModuleDestroy, onApplicationShutdown)
- Implement procedural macros: Injectable, Module, controller, routes, HTTP method attributes, parameter extractors
- Implement testing utilities: test module builder, provider overrides, in-process test application, fluent HTTP client
- Implement a CLI with scaffolding generators (project, module, controller, service, resource)
- Implement typed configuration, structured tracing, health endpoints, and security defaults

## Capabilities

### New Capabilities
- `dependency-injection`: Type-safe DI container with singleton/transient scopes, factory providers, value providers, cycle detection, and provider overrides
- `module-system`: Module graph compilation with imports, exports, provider visibility, circular import detection, and deterministic initialization order
- `http-routing`: Transport-neutral HTTP method/status/request/response types, route definitions, handler metadata, and parameter extraction contracts
- `axum-adapter`: Platform adapter converting compiled routes into an Axum router with DI-resolved controllers, Tower layers, and escape hatches
- `request-pipeline`: Deterministic request lifecycle — middleware chain, guard evaluation, interceptor before/after, parameter transformation, validation, and error propagation
- `lifecycle-hooks`: Module and application bootstrap/shutdown hooks with deterministic reverse-order cleanup
- `procedural-macros`: Injectable, Module, controller, routes, HTTP method, and parameter extractor derives
- `testing-utilities`: Test module compiler, provider/value/factory overrides, in-process test application, fluent HTTP request builder, and response assertions
- `cli-tooling`: Project scaffolding, code generators (module, controller, service, resource), build/start/test orchestration, doctor command
- `observability`: Tracing integration, request IDs, structured logging, health endpoints, metrics

## Impact

- Creates a multi-crate workspace under `ironic/` with 9 initial crates: common, di, http, platform, core, platform-axum, testing, macros, and facade
- Adds axum, tokio, tower, serde, thiserror, tracing, clap, syn, quote as core dependencies
- Introduces procedural macros that generate code calling public kernel APIs
- Provides a CLI that is purely an orchestration layer — the framework works without it
- All additions are additive — the framework sits on top of existing Rust ecosystem libraries
