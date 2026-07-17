---
title: "In-Process Testing Without a Socket — how TestApplication works"
description: "A deep dive into Ironic's zero-socket test harness: route matching, DI overrides, synchronous pipeline execution, and panic-safe cleanup."
date: "2026-07-15"
author: "Ironic Team"
---

# In-Process Testing Without a Socket — how TestApplication works

Integration tests that bind a TCP port are slow, flaky, and fight with CI port conflicts. Ironic's `TestApplication` solves this by running the **entire** framework pipeline — middleware, guards, interceptors, extraction, pipes, handler — directly in-process, without a socket, without an HTTP server, and without spawning a background Tokio runtime. The wire is never involved.

This post walks through every layer of the test harness: how it bypasses the platform adapter, how it matches routes by hand, how provider overrides replace real dependencies with mocks, and how cleanup runs correctly even when a test panics.

---

## The adapter that doesn't listen

Every Ironic application runs on a `HttpPlatformAdapter`. The production adapter (Axum) binds a TCP socket, constructs an Axum `Router`, and calls `axum::serve`. The test adapter at `application.rs:194-206` does none of that:

```rust
struct InProcessAdapter;

impl HttpPlatformAdapter for InProcessAdapter {
    type Application = InProcessApplication;
    type Error = Infallible;

    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<Self::Application, Self::Error> {
        Ok(InProcessApplication { application })
    }
}
```

`build()` doesn't create a router. It wraps `CompiledHttpApplication` — the framework's compiled route table, DI container, and pipeline components — in an inert `InProcessApplication` struct and returns it. No route translation, no method filter conversion, no Tower layers.

Then `listen()` at line 222:

```rust
impl HttpPlatformApplication for InProcessApplication {
    type Error = Infallible;

    fn listen(
        self,
        _address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>> {
        Box::pin(async move { Ok(shutdown.wait().await) })
    }
}
```

It ignores the address entirely. It returns *immediately* — or rather, it returns a future that resolves when the shutdown signal fires. In a normal test, this future is never even polled. `TestApplication` builds the framework through this adapter but calls `execute()` directly on the `CompiledHttpApplication`, completely bypassing `listen()`.

This is the architectural cheat code: because Ironic's HTTP pipeline is trait-abstracted behind `HttpPlatformAdapter`, swapping in a no-op adapter requires zero changes to core or HTTP crates.

---

## Route matching without a router

Since there's no Axum router to do path matching, `TestRequestBuilder::send()` at `request.rs:105-140` does it manually. When you write `app.get("/users/42").send().await`, here's what happens:

```rust
let route = self.application.routes().iter().find_map(|route| {
    (route.method() == self.method)
        .then(|| match_path(route.path(), request_path))
        .flatten()
        .map(|parameters| (route, parameters))
});
```

It iterates the compiled route table, filtering by HTTP method, then delegates to `match_path()`. This function at line 143 is a deterministic segment-by-segment matcher:

```rust
fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern = segments(pattern);
    let path = segments(path);
    if pattern.len() != path.len() {
        return None;
    }
    let mut parameters = HashMap::new();
    for (expected, actual) in pattern.into_iter().zip(path) {
        if let Some(name) = expected.strip_prefix(':') {
            parameters.insert(name.to_owned(), actual.to_owned());
        } else if expected != actual {
            return None;
        }
    }
    Some(parameters)
}
```

`segments()` splits on `/` and discards empty strings. For a pattern `/users/:id/posts/:post_id` and a path `/users/42/posts/7`, the first and third segments match literally, and `":id"` and `":post_id"` bind `"42"` and `"7"` respectively. No regex engine, no trie, no allocation beyond the HashMap — just a linear scan over slices.

If no route matches, `send()` returns a `404 Not Found` response with error code `RF_HTTP_ROUTE_NOT_FOUND`. The builder also eagerly validates headers and body serialization during construction (lines 40-55 and 60-75), returning structured errors before the pipeline even runs.

---

## Constructing the framework request and calling execute

Once a route is matched, `send()` at line 131 builds the request:

```rust
let request = Request::new(self.method, uri, self.headers, self.body)
    .with_path_parameters(parameters);
let mut context = RequestContext::new(request);
let response = self.application.execute(route, &mut context).await
    .unwrap_or_else(error_response);
TestResponse::new(response)
```

`Request::with_path_parameters()` attaches the extracted `{id: "42"}` map. Then `RequestContext::new()` creates a fresh context — extensions, route metadata, and DI scope are all initially empty. The call to `CompiledHttpApplication::execute()` is the same entry point the production adapter uses. From here, the entire pipeline runs:

- **Middleware** receives `context: &mut RequestContext` and a `MiddlewareNext` handle. Global, controller, and route-level middleware fire in order.
- **Guards** iterate and call `can_activate(context)`. Any `Deny` short-circuits the pipeline with a forbidden error, and the middleware onion unwinds in reverse.
- **Interceptors** wrap the handler call. At the base of the interceptor chain, the controller is resolved from the DI container's request scope, and `invoke_handler()` runs.
- **Extraction** iterates over declared parameters — `PathParameter<u64>` reads `"42"` from the path parameters map and parses it.
- **Pipes** transform extracted values. Global pipes run first, then controller, then route-level.
- **Handler** calls your closure with the typed controller and extracted arguments.

Everything runs synchronously within the test's Tokio runtime — the same thread that called `send().await`. There's no socket read, no byte serialization, no network round-trip. The request body is already in memory as `Vec<u8>`. The response body comes back as `Response` with no serialization overhead.

---

## TestResponse: assertions that don't lie

`TestResponse` at `response.rs` wraps `Response` and provides a focused assertion API:

- **`.assert_status(200)`** — compares against the actual `HttpStatus::as_u16()`, panicking with the response body in the message so you can see what went wrong.
- **`.assert_json(&expected)`** — deserializes the response body as `serde_json::Value`, serializes the expected value, and compares structurally. This avoids false positives from field ordering differences.
- **`.assert_error("RF_HTTP_ROUTE_NOT_FOUND")`** — deserializes the body as JSON and checks `body["code"]`. This is how you assert that guards, pipes, and exception filters produce the right error codes.
- **`.assert_header("content-type", "application/json")`** — single-header assertion with a clear panic message when the header is missing.
- **`.status()`**, **`.headers()`**, **`.json::<T>()`**, **`.body()`** — accessor methods for when you want to inspect the response without panicking.

Every assertion panics with a descriptive message. In a test harness, a panic is the right failure mode — it stops the test immediately, produces a clear diff, and doesn't require you to pattern-match on `Result` variants.

---

## TestModule: the DI graph without HTTP

Sometimes you don't need the HTTP pipeline at all. You just want to verify that a module graph compiles correctly and that providers resolve. `TestModule` at `module.rs:12` is a stripped-down compiler:

```rust
pub async fn compile(self) -> Result<CompiledTestModule, TestBuildError> {
    let graph = compile_module_graph(self.root)?;
    let application = build_http_application_with_overrides(&graph, self.overrides)?;
    let container = application.container().clone();
    for module_id in graph.initialization_order() {
        // ... resolve eager providers
    }
    Ok(CompiledTestModule { graph, container })
}
```

It compiles the module graph, applies overrides, builds an HTTP application (for DI container initialization only), then eagerly resolves every eager provider in topological order. The resulting `CompiledTestModule` exposes `.graph()` for structural assertions and `.resolve::<T>()` for provider access — no routes, no middleware, no HTTP.

---

## Provider overrides: swapping the world

Both `TestApplicationBuilder` and `TestModuleBuilder` support three override methods:

- **`override_provider(ProviderDefinition)`** — injects a complete provider definition (with its own key, scope, dependencies, and factory). This is the most general form.
- **`override_value(my_mock)`** — shorthand for a singleton value. Internally calls `ProviderDefinition::value(value)`.
- **`override_factory::<T>(scope, deps, |resolver| async { ... })`** — shorthand for an async factory with explicit dependencies. The closure receives a `Resolver` and returns `Result<T, ResolveError>`.

Overrides are collected into a `Vec<ProviderDefinition>` during builder construction, then passed to `Application::builder().override_provider()` or `build_http_application_with_overrides()` during compilation. The DI container merges them into the provider graph — test overrides take precedence over module-registered providers of the same concrete type. This means you can replace a database connection pool with an in-memory mock without touching a line of production code.

---

## The Drop guarantee: cleanup in a separate runtime

The most subtle feature is in `TestApplication::Drop` at `application.rs:162-192`:

```rust
impl Drop for TestApplication {
    fn drop(&mut self) {
        let Some(application) = self.application.take() else {
            return;
        };
        let cleanup = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Ironic test cleanup runtime must initialize")
                .block_on(application.shutdown(ShutdownSignal::Custom("test-drop")))
        })
        .join();

        if std::thread::panicking() {
            return;
        }
        // ... panic on cleanup failure
    }
}
```

When a test panics, the Tokio runtime's state is undefined — in-flight tasks may be cancelled, resources may be half-dropped. Running shutdown hooks on the panicked runtime would deadlock or double-panic. Instead, `Drop` spawns a **brand-new OS thread** with a fresh single-threaded Tokio runtime and runs `shutdown()` there. This guarantees that `application_shutdown` and `module_destroy` lifecycle hooks run in a clean environment.

If the test completed normally (no panic), `std::thread::panicking()` returns `false` and the code checks whether cleanup succeeded. If cleanup failed or panicked, `Drop` panics explicitly — because a test that leaks resources or fails to shut down is a test that deserves to fail.

The `application.take()` call ensures shutdown runs at most once. If you called `shutdown()` explicitly (which also takes the `Option`), `Drop` sees `None` and returns immediately. This makes cleanup idempotent — you can shut down manually or rely on `Drop`, but not double-shutdown.

---

## Summary

Ironic's test harness works because the framework architecture treats the HTTP runtime as a pluggable detail. By swapping in `InProcessAdapter`, the test layer inherits the entire pipeline — middleware, guards, interceptors, extraction, pipes, and handlers — without a socket, without serialization overhead, and without a background server. Route matching is a deterministic segment-by-segment scan. Provider overrides let you mock anything in the DI graph. And `Drop` ensures cleanup runs even when the test panics, by spawning a separate OS thread with a fresh runtime.

The result: integration tests that feel like unit tests — fast, deterministic, and panic-safe.
