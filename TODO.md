# RustFrame Implementation TODO

This checklist turns the architecture in [`rust_framework_full_docs.md`](./rust_framework_full_docs.md) into an implementation sequence. Complete phases in order unless a task is explicitly marked as parallelizable.

## Phase 0 — Architecture decisions

- [ ] Write RFC 0001: module identity, imports, exports, visibility, and initialization order.
- [ ] Write RFC 0002: DI tokens, singleton/transient scopes, factories, overrides, and cycle detection.
- [ ] Decide whether provider construction is synchronous, asynchronous, or represented by separate provider traits.
- [ ] Define the supported MVP strategy for trait-object dependencies.
- [ ] Write RFC 0003: type-erased controller handlers, route metadata, and parameter extraction.
- [ ] Write RFC 0004: middleware, guard, interceptor, validation, and error execution order.
- [ ] Write RFC 0005: platform adapter boundary and Axum/Tower escape hatches.
- [ ] Create compile-only API sketches for one module, provider, controller, and route.

**Exit gate:** foundational contracts are documented and the explicit public API is coherent without procedural macros.

## Phase 1 — Workspace foundation

- [ ] Create the root Cargo workspace.
- [ ] Add initial crates:
  - [ ] `rustframe-common`
  - [ ] `rustframe-di`
  - [ ] `rustframe-http`
  - [ ] `rustframe-platform`
  - [ ] `rustframe-core`
  - [ ] `rustframe-platform-axum`
  - [ ] `rustframe-testing`
  - [ ] `rustframe-macros`
  - [ ] `rustframe` facade
- [ ] Define and document allowed crate dependency directions.
- [ ] Centralize workspace dependencies, lint settings, and package metadata.
- [ ] Select and document the minimum supported Rust version.
- [ ] Add formatting, Clippy, test, documentation, audit, and dependency-policy checks.
- [ ] Add CI for the supported Rust toolchain.
- [ ] Add a minimal `examples/hello-world` package.

**Exit gate:** all crates build independently, workspace checks pass, and no platform-specific dependency leaks into neutral crates.

## Phase 2 — Dependency injection kernel

- [ ] Define typed provider keys and registration metadata.
- [ ] Implement concrete-type provider registration.
- [ ] Implement singleton resolution with concurrency-safe one-time initialization.
- [ ] Implement transient resolution.
- [ ] Implement value providers.
- [ ] Implement factory providers according to the approved sync/async contract.
- [ ] Implement explicit wrapper-token support for trait dependencies.
- [ ] Implement resolution-path tracking.
- [ ] Report missing, duplicate, downcast, and construction errors with actionable messages.
- [ ] Detect circular dependency resolution deterministically.
- [ ] Implement provider overrides without global mutable state.
- [ ] Add unit and concurrency tests for every provider mode and error path.

**Exit gate:** singleton, transient, factory, error-chain, cycle, and override behavior is deterministic and fully tested.

## Phase 3 — Module compiler

- [ ] Define stable module and compiled-module representations.
- [ ] Traverse imports and deduplicate modules.
- [ ] Detect circular module imports.
- [ ] Assign providers and controllers to owning modules.
- [ ] Validate provider exports.
- [ ] Enforce cross-module provider visibility.
- [ ] Compute deterministic initialization and shutdown order.
- [ ] Produce a compiled application graph for runtime use and diagnostics.
- [ ] Add fixtures covering valid graphs, duplicates, cycles, invalid exports, and private access.

**Exit gate:** a multi-module application compiles into a validated, deterministic graph with actionable failures.

## Phase 4 — HTTP contracts and Axum vertical slice

- [ ] Define transport-neutral HTTP methods, statuses, requests, responses, and headers.
- [ ] Define `IntoFrameworkResponse` and structured rejection/error contracts.
- [ ] Define executable, type-erased route and handler representations.
- [ ] Define JSON body, path, and query extraction contracts.
- [ ] Define platform adapter traits.
- [ ] Implement conversion from compiled routes to an Axum router.
- [ ] Resolve controller instances through DI.
- [ ] Invoke controller handlers through the erased handler contract.
- [ ] Convert framework responses and errors into Axum responses.
- [ ] Expose Tower layer and raw Axum router escape hatches.
- [ ] Build an explicit-API `GET /users/:id` end-to-end example.
- [ ] Add in-process integration tests for success, malformed input, and not-found behavior.

**Exit gate:** a real Axum request reaches a DI-managed controller and service using explicit Rust APIs and returns a structured response.

## Phase 5 — Request pipeline

- [ ] Implement the framework middleware chain.
- [ ] Implement guard evaluation and denial mapping.
- [ ] Implement parameter transformation and validation.
- [ ] Implement nested interceptor before/after behavior.
- [ ] Implement application, controller, and route-level component registration.
- [ ] Define and implement error propagation at every pipeline stage.
- [ ] Decide whether the MVP includes panic isolation; document the boundary either way.
- [ ] Add ordering tests for success and every failure stage.

**Exit gate:** pipeline ordering matches the documented lifecycle and remains stable under success and failure.

## Phase 6 — Application lifecycle

- [ ] Implement `FrameworkApplication::builder()` with an explicit root module.
- [ ] Compile module and provider graphs during application build.
- [ ] Instantiate eager singleton providers.
- [ ] Implement module initialization hooks.
- [ ] Implement application bootstrap hooks.
- [ ] Build and start the selected platform adapter.
- [ ] Implement graceful shutdown signal handling.
- [ ] Run destruction and shutdown hooks in deterministic reverse order.
- [ ] Clean up partially initialized applications after startup failure.
- [ ] Add lifecycle order, failure, and graceful-shutdown tests.

**Exit gate:** an explicit-API application starts, serves requests, and shuts down cleanly with verified hook order.

## Phase 7 — Procedural macros

- [ ] Implement `Injectable` derive.
- [ ] Implement `Module` derive and module metadata attributes.
- [ ] Implement `controller` and `routes` attributes.
- [ ] Implement HTTP method attributes.
- [ ] Implement `body`, `query`, `param`, and header parameter attributes.
- [ ] Implement guard and interceptor attributes.
- [ ] Implement `rustframe::main` only after bootstrap APIs stabilize.
- [ ] Ensure generated code calls public kernel APIs and contains no independent runtime behavior.
- [ ] Add `trybuild` compile-pass and compile-fail coverage with precise diagnostics.
- [ ] Rebuild the explicit example with macros and verify identical behavior.

**Exit gate:** macro and explicit applications are behaviorally equivalent, and invalid usage produces useful compiler diagnostics.

## Phase 8 — Testing utilities

- [ ] Implement a test module compiler.
- [ ] Support provider, value, and factory overrides.
- [ ] Implement an in-process test application without binding a network port.
- [ ] Add a fluent HTTP request builder.
- [ ] Add status, header, JSON, and error response assertions.
- [ ] Ensure lifecycle cleanup runs after every test application.
- [ ] Document unit-testing services and integration-testing controllers.

**Exit gate:** users can replace dependencies and test a complete request without global state or a real socket.

## Phase 9 — Minimum viable CLI

- [ ] Create the `rustframe-cli` crate and command structure.
- [ ] Implement `rustframe new`.
- [ ] Implement `start`, `build`, and `test` as transparent Cargo orchestration.
- [ ] Implement module generation.
- [ ] Implement controller generation.
- [ ] Implement service generation.
- [ ] Implement resource generation.
- [ ] Implement a minimal `doctor` command.
- [ ] Make generators deterministic and idempotent.
- [ ] Print manual registration instructions when source modification is unsafe.
- [ ] Add syntax-aware module registration only after its safety is covered by tests.
- [ ] Test generated projects by building and running their test suites.

**Exit gate:** a generated project builds and tests without manual repair, and repeated generation does not corrupt source files.

## Phase 10 — Version 0.1 hardening

- [ ] Add typed configuration with validation and secret redaction.
- [ ] Add structured `tracing` integration and request IDs.
- [ ] Add health endpoint support.
- [ ] Define safe request-size and timeout defaults.
- [ ] Document CORS, security headers, rate limiting integration, and secret handling.
- [ ] Benchmark startup, route registration, provider resolution, and request overhead against equivalent raw Axum applications.
- [ ] Add getting-started and fundamentals documentation.
- [ ] Add hello-world, REST API, validation, error handling, and testing examples.
- [ ] Document every public API and enable missing-doc linting where appropriate.
- [ ] Run formatting, Clippy, all-feature tests, docs, macro UI tests, audit, and dependency-policy checks.
- [ ] Publish release notes with known limitations and benchmark results.

**Exit gate:** the documented 0.1 feature set is tested, usable without the CLI, observable, secure by default, and supported by complete examples.

## Deferred until after 0.1

- [ ] OpenAPI generation and Swagger UI.
- [ ] SQLx, SeaORM, Diesel, MongoDB, and Redis integration crates.
- [ ] Authentication, JWT, OAuth, sessions, and authorization helpers.
- [ ] Request-scoped providers.
- [ ] Dynamic, lazy, conditional, and async-configured modules.
- [ ] Route inspector and dependency graph CLI commands.
- [ ] Caching, scheduling, events, WebSockets, and SSE.
- [ ] Queues, microservice transports, gRPC, CQRS, sagas, and GraphQL.
- [ ] Devtools UI and plugin ecosystem.

## Version 0.1 release checklist

- [ ] A non-trivial REST application works with modules, DI, controllers, and macros.
- [ ] The same application can be expressed through explicit APIs.
- [ ] Module and provider errors include actionable dependency paths.
- [ ] Request-pipeline ordering is deterministic and tested.
- [ ] Testing overrides use no global state.
- [ ] Raw Axum and Tower extensions remain accessible.
- [ ] CLI-generated projects build without manual fixes.
- [ ] Public APIs include examples and error documentation.
- [ ] CI and security checks pass on the supported toolchain.
- [ ] Framework overhead is measured and published.
