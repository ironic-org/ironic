## 1. Workspace Foundation

- [x] 1.1 Create the root Cargo workspace with dependency and lint configuration
- [x] 1.2 Add initial crates: common, di, http, platform, core, platform-axum, testing, macros, facade
- [x] 1.3 Add formatting, Clippy, test, documentation, audit, and dependency-policy checks
- [x] 1.4 Add CI for the supported Rust toolchain
- [x] 1.5 Add a minimal `examples/hello-world` package

## 2. Dependency Injection Kernel

- [x] 2.1 Define typed provider keys and registration metadata
- [x] 2.2 Implement concrete-type provider registration
- [x] 2.3 Implement singleton resolution with concurrency-safe one-time initialization
- [x] 2.4 Implement transient resolution
- [x] 2.5 Implement value providers
- [x] 2.6 Implement factory providers
- [x] 2.7 Implement explicit wrapper-token support for trait dependencies
- [x] 2.8 Implement resolution-path tracking with actionable error messages
- [x] 2.9 Detect circular dependency resolution deterministically
- [x] 2.10 Implement provider overrides without global mutable state
- [x] 2.11 Add unit and concurrency tests for every provider mode and error path

## 3. Module Compiler

- [x] 3.1 Define stable module and compiled-module representations
- [x] 3.2 Traverse imports and deduplicate modules
- [x] 3.3 Detect circular module imports
- [x] 3.4 Assign providers and controllers to owning modules
- [x] 3.5 Validate provider exports and enforce cross-module visibility
- [x] 3.6 Compute deterministic initialization and shutdown order
- [x] 3.7 Produce a compiled application graph for runtime use and diagnostics
- [x] 3.8 Add fixtures covering valid graphs, duplicates, cycles, invalid exports, and private access

## 4. HTTP Contracts and Axum Adapter

- [x] 4.1 Define transport-neutral HTTP methods, statuses, requests, responses, and headers
- [x] 4.2 Define IntoFrameworkResponse and structured rejection/error contracts
- [x] 4.3 Define executable, type-erased route and handler representations
- [x] 4.4 Define JSON body, path, and query extraction contracts
- [x] 4.5 Define platform adapter traits
- [x] 4.6 Implement conversion from compiled routes to an Axum router
- [x] 4.7 Resolve controller instances through DI
- [x] 4.8 Invoke controller handlers through the erased handler contract
- [x] 4.9 Convert framework responses and errors into Axum responses
- [x] 4.10 Expose Tower layer and raw Axum router escape hatches
- [x] 4.11 Build an explicit-API GET /users/:id end-to-end example
- [x] 4.12 Add in-process integration tests for success, malformed input, and not-found behavior

## 5. Request Pipeline

- [x] 5.1 Implement the framework middleware chain
- [x] 5.2 Implement guard evaluation and denial mapping
- [x] 5.3 Implement parameter transformation and validation
- [x] 5.4 Implement nested interceptor before/after behavior
- [x] 5.5 Implement application, controller, and route-level component registration
- [x] 5.6 Define and implement error propagation at every pipeline stage
- [x] 5.7 Add ordering tests for success and every failure stage

## 6. Application Lifecycle

- [x] 6.1 Implement FrameworkApplication::builder() with an explicit root module
- [x] 6.2 Compile module and provider graphs during application build
- [x] 6.3 Instantiate eager singleton providers
- [x] 6.4 Implement module initialization hooks
- [x] 6.5 Implement application bootstrap hooks
- [x] 6.6 Build and start the selected platform adapter
- [x] 6.7 Implement graceful shutdown signal handling
- [x] 6.8 Run destruction and shutdown hooks in deterministic reverse order
- [x] 6.9 Clean up partially initialized applications after startup failure
- [x] 6.10 Add lifecycle order, failure, and graceful-shutdown tests

## 7. Procedural Macros

- [x] 7.1 Implement Injectable derive
- [x] 7.2 Implement Module derive and module metadata attributes
- [x] 7.3 Implement controller and routes attributes
- [x] 7.4 Implement HTTP method attributes
- [x] 7.5 Implement body, query, param, and header parameter attributes
- [x] 7.6 Implement guard and interceptor attributes
- [x] 7.7 Implement rustframe::main bootstrap macro
- [x] 7.8 Ensure generated code calls public kernel APIs with no independent runtime behavior
- [x] 7.9 Add trybuild compile-pass and compile-fail coverage with precise diagnostics
- [x] 7.10 Rebuild explicit example with macros and verify identical behavior

## 8. Testing Utilities

- [x] 8.1 Implement a test module compiler
- [x] 8.2 Support provider, value, and factory overrides
- [x] 8.3 Implement an in-process test application without binding a network port
- [x] 8.4 Add a fluent HTTP request builder
- [x] 8.5 Add status, header, JSON, and error response assertions
- [x] 8.6 Ensure lifecycle cleanup runs after every test application
- [x] 8.7 Document unit-testing services and integration-testing controllers

## 9. Minimum Viable CLI

- [x] 9.1 Create the CLI command structure
- [x] 9.2 Implement rustframe new project scaffolding
- [x] 9.3 Implement start, build, and test as transparent Cargo orchestration
- [x] 9.4 Implement module, controller, service, and resource generation
- [x] 9.5 Implement a minimal doctor command
- [x] 9.6 Make generators deterministic and idempotent
- [x] 9.7 Print manual registration instructions when source modification is unsafe
- [x] 9.8 Test generated projects by building and running their test suites

## 10. Version 0.1 Hardening

- [x] 10.1 Add typed configuration with validation and secret redaction
- [x] 10.2 Add structured tracing integration and request IDs
- [x] 10.3 Add health endpoint support
- [x] 10.4 Define safe request-size and timeout defaults
- [x] 10.5 Document CORS, security headers, rate limiting integration, and secret handling
- [x] 10.6 Benchmark startup, route registration, provider resolution, and request overhead
- [x] 10.7 Add getting-started and fundamentals documentation
- [x] 10.8 Add hello-world, REST API, validation, error handling, and testing examples
- [x] 10.9 Document every public API and enable missing-doc linting where appropriate
- [x] 10.10 Run formatting, Clippy, all-feature tests, docs, macro UI tests, audit, and dependency-policy checks
- [x] 10.11 Publish release notes with known limitations and benchmark results
