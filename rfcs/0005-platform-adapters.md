# RFC 0005: Platform Adapter Boundary

- Status: Accepted for initial implementation
- Target: Ironic 0.1

## Summary

The kernel compiles transport-neutral application metadata into a `CompiledHttpApplication`. A platform adapter consumes that immutable description, converts native requests into framework request contexts, executes the framework pipeline, and converts framework responses back to native responses. Axum is the only 0.1 adapter.

## Dependency boundary

```text
ironic-common       ironic-di
         \               /
          ironic-core
               │
       ironic-http
               │
     ironic-platform
               │
 ironic-platform-axum ──▶ axum / tower / hyper / tokio
```

Neutral crates must not depend on Axum, Tower, or Hyper types. Tokio may be used by runtime and DI implementations but must not appear in transport-neutral public contracts unless the contract is explicitly runtime-specific.

## Adapter contract

```rust
pub trait HttpPlatformAdapter: Send + 'static {
    type NativeRouter;
    type Error: std::error::Error + Send + Sync + 'static;

    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<PlatformApplication<Self::NativeRouter>, Self::Error>;
}
```

Building route tables is synchronous because it performs no I/O. Listening and shutdown belong to `PlatformApplication`:

```rust
pub trait HttpPlatformApplication: Send + 'static {
    fn listen(
        self,
        address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<(), PlatformError>>;
}
```

The concrete Axum application may expose additional native methods without adding them to the neutral trait.

## Request conversion

The Axum adapter:

1. Matches a native request using generated Axum routes.
2. Applies configured Tower layers.
3. Captures native path parameters and request metadata.
4. Enforces configured body-size limits while converting the body lazily or eagerly according to the HTTP implementation.
5. Creates the owned `RequestContext`.
6. Executes the compiled framework pipeline.
7. Converts the framework response to an Axum response.

Native extensions that are `Send + Sync + 'static` may be copied or moved into the framework extension map through explicit adapter bridges. The adapter does not attempt to clone arbitrary native extensions.

## Tower escape hatch

The Axum application builder accepts Tower layers before final router construction:

```rust
let app = Application::builder()
    .module(AppModule::definition())
    .platform(
        AxumAdapter::new()
            .layer(TraceLayer::new_for_http())
            .layer(RequestBodyLimitLayer::new(1_048_576)),
    )
    .build()
    .await?;
```

Layer ordering follows Tower conventions and is documented by the Axum adapter. Layers wrap framework routes and native escape-hatch routes alike.

## Router escape hatch

Native routes are added before the adapter is finalized:

```rust
AxumAdapter::new().configure_router(|router| {
    router.route("/native", axum::routing::get(native_handler))
})
```

Rules:

- A native route conflicting with a framework route is a startup error when detectable; otherwise Axum's route-conflict error is wrapped.
- Native routes participate in configured Tower layers.
- Native routes do not automatically receive framework middleware, guards, interceptors, validation, DI controllers, or route metadata.
- Applications may invoke public Ironic services manually from native state if they explicitly attach that state.

The closure is called once during application build. The running router is immutable.

## Native state

The adapter owns one internal state object containing the compiled application and container handles. User native state is composed through an explicit wrapper or extension API; Ironic does not overwrite existing state silently.

## Listening and shutdown

- Address parsing happens before binding.
- Bind errors retain the address and underlying I/O error.
- Shutdown is driven by a framework `Shutdown` future/token supplied to the adapter.
- The adapter stops accepting connections, waits according to configured graceful-shutdown policy, and then returns control so framework shutdown hooks can run.
- Tests build the router and call it in-process without binding a socket.

## Errors

Platform errors are wrapped by `AppError::Platform` and use stable categories: route conflict, request conversion, response conversion, bind, serve, and graceful shutdown. Client responses never include raw I/O or native framework errors.

## Alternatives considered

- An async route-registration method was rejected because route construction is deterministic startup work without I/O.
- Exposing Axum request/response types in `ironic-http` was rejected because it would make the adapter abstraction cosmetic.
- Automatically wrapping native routes in the framework pipeline was rejected because parameter and controller metadata would be absent.

## Performance impact

The adapter adds request/response conversion and one application-state lookup. Route metadata and pipeline lists are immutable and shared. Benchmarks compare equivalent raw Axum and Ironic applications.

## Testing strategy

- Compile checks preventing Axum dependencies in neutral crates.
- Route and response conversion tests.
- Body limit and malformed request tests.
- Tower layer ordering tests.
- Native router escape-hatch tests.
- Conflict, bind, and shutdown error tests.
- In-process router integration tests.

## Unresolved questions

None for initial implementation. Additional adapters must prove that the neutral contracts are sufficient before shared abstractions are expanded.
