# RustFrame Implementation TODO

This checklist turns the architecture in [`rust_framework_full_docs.md`](./rust_framework_full_docs.md) into an implementation sequence. Complete phases in order unless a task is explicitly marked as parallelizable.

## Phase 0 — Architecture decisions

- [x] Write RFC 0001: module identity, imports, exports, visibility, and initialization order.
- [x] Write RFC 0002: DI tokens, singleton/transient scopes, factories, overrides, and cycle detection.
- [x] Decide whether provider construction is synchronous, asynchronous, or represented by separate provider traits.
- [x] Define the supported MVP strategy for trait-object dependencies.
- [x] Write RFC 0003: type-erased controller handlers, route metadata, and parameter extraction.
- [x] Write RFC 0004: middleware, guard, interceptor, validation, and error execution order.
- [x] Write RFC 0005: platform adapter boundary and Axum/Tower escape hatches.
- [x] Create compile-only API sketches for one module, provider, controller, and route.

**Exit gate:** foundational contracts are documented and the explicit public API is coherent without procedural macros.

## Phase 1 — Workspace foundation

- [x] Create the root Cargo workspace.
- [x] Add initial crates:
  - [x] `rustframe-common`
  - [x] `rustframe-di`
  - [x] `rustframe-http`
  - [x] `rustframe-platform`
  - [x] `rustframe-core`
  - [x] `rustframe-platform-axum`
  - [x] `rustframe-testing`
  - [x] `rustframe-macros`
  - [x] `rustframe` facade
- [x] Define and document allowed crate dependency directions.
- [x] Centralize workspace dependencies, lint settings, and package metadata.
- [x] Select and document the minimum supported Rust version.
- [x] Add formatting, Clippy, test, documentation, audit, and dependency-policy checks.
- [x] Add CI for the supported Rust toolchain.
- [x] Add a minimal `examples/hello-world` package.

**Exit gate:** all crates build independently, workspace checks pass, and no platform-specific dependency leaks into neutral crates.

## Phase 2 — Dependency injection kernel

- [x] Define typed provider keys and registration metadata.
- [x] Implement concrete-type provider registration.
- [x] Implement singleton resolution with concurrency-safe one-time initialization.
- [x] Implement transient resolution.
- [x] Implement value providers.
- [x] Implement factory providers according to the approved sync/async contract.
- [x] Implement explicit wrapper-token support for trait dependencies.
- [x] Implement resolution-path tracking.
- [x] Report missing, duplicate, downcast, and construction errors with actionable messages.
- [x] Detect circular dependency resolution deterministically.
- [x] Implement provider overrides without global mutable state.
- [x] Add unit and concurrency tests for every provider mode and error path.

**Exit gate:** singleton, transient, factory, error-chain, cycle, and override behavior is deterministic and fully tested.

## Phase 3 — Module compiler

- [x] Define stable module and compiled-module representations.
- [x] Traverse imports and deduplicate modules.
- [x] Detect circular module imports.
- [x] Assign providers and controllers to owning modules.
- [x] Validate provider exports.
- [x] Enforce cross-module provider visibility.
- [x] Compute deterministic initialization and shutdown order.
- [x] Produce a compiled application graph for runtime use and diagnostics.
- [x] Add fixtures covering valid graphs, duplicates, cycles, invalid exports, and private access.

**Exit gate:** a multi-module application compiles into a validated, deterministic graph with actionable failures.

## Phase 4 — HTTP contracts and Axum vertical slice

- [x] Define transport-neutral HTTP methods, statuses, requests, responses, and headers.
- [x] Define `IntoFrameworkResponse` and structured rejection/error contracts.
- [x] Define executable, type-erased route and handler representations.
- [x] Define JSON body, path, and query extraction contracts.
- [x] Define platform adapter traits.
- [x] Implement conversion from compiled routes to an Axum router.
- [x] Resolve controller instances through DI.
- [x] Invoke controller handlers through the erased handler contract.
- [x] Convert framework responses and errors into Axum responses.
- [x] Expose Tower layer and raw Axum router escape hatches.
- [x] Build an explicit-API `GET /users/:id` end-to-end example.
- [x] Add in-process integration tests for success, malformed input, and not-found behavior.

**Exit gate:** a real Axum request reaches a DI-managed controller and service using explicit Rust APIs and returns a structured response.

## Phase 5 — Request pipeline

- [x] Implement the framework middleware chain.
- [x] Implement guard evaluation and denial mapping.
- [x] Implement parameter transformation and validation.
- [x] Implement nested interceptor before/after behavior.
- [x] Implement application, controller, and route-level component registration.
- [x] Define and implement error propagation at every pipeline stage.
- [x] Decide whether the MVP includes panic isolation; document the boundary either way.
- [x] Add ordering tests for success and every failure stage.

**Exit gate:** pipeline ordering matches the documented lifecycle and remains stable under success and failure.

## Phase 6 — Application lifecycle

- [x] Implement `FrameworkApplication::builder()` with an explicit root module.
- [x] Compile module and provider graphs during application build.
- [x] Instantiate eager singleton providers.
- [x] Implement module initialization hooks.
- [x] Implement application bootstrap hooks.
- [x] Build and start the selected platform adapter.
- [x] Implement graceful shutdown signal handling.
- [x] Run destruction and shutdown hooks in deterministic reverse order.
- [x] Clean up partially initialized applications after startup failure.
- [x] Add lifecycle order, failure, and graceful-shutdown tests.

**Exit gate:** an explicit-API application starts, serves requests, and shuts down cleanly with verified hook order.

## Phase 7 — Procedural macros

- [x] Implement `Injectable` derive.
- [x] Implement `Module` derive and module metadata attributes.
- [x] Implement `controller` and `routes` attributes.
- [x] Implement HTTP method attributes.
- [x] Implement `body`, `query`, `param`, and header parameter attributes.
- [x] Implement guard and interceptor attributes.
- [x] Implement `rustframe::main` only after bootstrap APIs stabilize.
- [x] Ensure generated code calls public kernel APIs and contains no independent runtime behavior.
- [x] Add `trybuild` compile-pass and compile-fail coverage with precise diagnostics.
- [x] Rebuild the explicit example with macros and verify identical behavior.

**Exit gate:** macro and explicit applications are behaviorally equivalent, and invalid usage produces useful compiler diagnostics.

## Phase 8 — Testing utilities

- [x] Implement a test module compiler.
- [x] Support provider, value, and factory overrides.
- [x] Implement an in-process test application without binding a network port.
- [x] Add a fluent HTTP request builder.
- [x] Add status, header, JSON, and error response assertions.
- [x] Ensure lifecycle cleanup runs after every test application.
- [x] Document unit-testing services and integration-testing controllers.

**Exit gate:** users can replace dependencies and test a complete request without global state or a real socket.

## Phase 9 — Minimum viable CLI

- [x] Create the `rustframe-cli` crate and command structure.
- [x] Implement `rustframe new`.
- [x] Implement `start`, `build`, and `test` as transparent Cargo orchestration.
- [x] Implement module generation.
- [x] Implement controller generation.
- [x] Implement service generation.
- [x] Implement resource generation.
- [x] Implement a minimal `doctor` command.
- [x] Make generators deterministic and idempotent.
- [x] Print manual registration instructions when source modification is unsafe.
- [x] Add syntax-aware module registration only after its safety is covered by tests.
- [x] Test generated projects by building and running their test suites.

**Exit gate:** a generated project builds and tests without manual repair, and repeated generation does not corrupt source files.

## Phase 10 — Version 0.1 hardening

- [x] Add typed configuration with validation and secret redaction.
- [x] Add structured `tracing` integration and request IDs.
- [x] Add health endpoint support.
- [x] Define safe request-size and timeout defaults.
- [x] Document CORS, security headers, rate limiting integration, and secret handling.
- [x] Benchmark startup, route registration, provider resolution, and request overhead against equivalent raw Axum applications.
- [x] Add getting-started and fundamentals documentation.
- [x] Add hello-world, REST API, validation, error handling, and testing examples.
- [x] Document every public API and enable missing-doc linting where appropriate.
- [x] Run formatting, Clippy, all-feature tests, docs, macro UI tests, audit, and dependency-policy checks.
- [x] Publish release notes with known limitations and benchmark results.

**Exit gate:** the documented 0.1 feature set is tested, usable without the CLI, observable, secure by default, and supported by complete examples.

## Deferred until after 0.1

- [x] OpenAPI generation and Swagger UI.
- [ ] SQLx, SeaORM, Diesel, MongoDB, and Redis integration crates.
- [ ] Authentication, JWT, OAuth, sessions, and authorization helpers.
- [ ] Request-scoped providers.
- [ ] Dynamic, lazy, conditional, and async-configured modules.
- [ ] Route inspector and dependency graph CLI commands.
- [ ] Caching, scheduling, events, WebSockets, and SSE.
- [ ] Queues, microservice transports, gRPC, CQRS, sagas, and GraphQL.
- [ ] Devtools UI and plugin ecosystem.

## Version 0.1 release checklist

- [x] A non-trivial REST application works with modules, DI, controllers, and macros.
- [x] The same application can be expressed through explicit APIs.
- [x] Module and provider errors include actionable dependency paths.
- [x] Request-pipeline ordering is deterministic and tested.
- [x] Testing overrides use no global state.
- [x] Raw Axum and Tower extensions remain accessible.
- [x] CLI-generated projects build without manual fixes.
- [x] Public APIs include examples and error documentation.
- [x] CI and security checks pass on the supported toolchain.
- [x] Framework overhead is measured and published.
