# RFC 0003: Controllers and Routing

- Status: Accepted for initial implementation
- Target: RustFrame 0.1

## Summary

Controllers are DI-managed concrete types. Routes store static metadata plus a type-erased asynchronous handler that receives a resolved controller and a transport-neutral request context. Parameter extractors are ordered, type-erased operations whose results are downcast by generated or handwritten handler adapters.

## Goals

- Keep controller methods ordinary async Rust methods.
- Make explicit definitions and macro-generated definitions equivalent.
- Erase heterogeneous controller signatures behind one platform-neutral contract.
- Validate route shape before the server starts.
- Preserve structured extraction and handler errors.

## Non-goals

- Runtime method reflection.
- Arbitrary Axum extractor support in transport-neutral handlers.
- Streaming, multipart, WebSockets, SSE, versioning, or host routing in 0.1.
- Inferring routes by scanning source files.

## Definitions

```rust
pub struct ControllerDefinition {
    pub key: ProviderKey,
    pub type_name: &'static str,
    pub path: &'static str,
    pub routes: Vec<RouteDefinition>,
    pub dependencies: Vec<Dependency>,
    pub factory: Arc<dyn ProviderFactory>,
}

pub struct RouteDefinition {
    pub method: HttpMethod,
    pub path: &'static str,
    pub handler_name: &'static str,
    pub handler: Arc<dyn ErasedHandler>,
    pub parameters: Vec<ParameterDefinition>,
    pub metadata: MetadataMap,
}
```

Controller keys use concrete controller types. Controllers are constructed through the DI engine after their owning module's eager providers, but are not exportable in 0.1.

## Type-erased handlers

```rust
pub type HandlerFuture<'a> = Pin<Box<
    dyn Future<Output = Result<FrameworkResponse, HandlerError>> + Send + 'a
>>;

pub trait ErasedHandler: Send + Sync + 'static {
    fn call<'a>(
        &'a self,
        controller: Arc<dyn Any + Send + Sync>,
        context: &'a mut RequestContext,
    ) -> HandlerFuture<'a>;
}
```

An adapter generated for a controller method performs these steps:

1. Downcast the controller to the declared concrete type.
2. Run parameter extractors in declaration order.
3. Downcast each extracted value to the method parameter type.
4. Invoke the method.
5. Convert the result through `IntoFrameworkResponse`.

Downcast failure is an internal framework-definition error, not a client error.

Explicit definitions use the same adapter helper API:

```rust
RouteDefinition::get("/:id")
    .parameter(ParameterDefinition::path::<UserId>("id"))
    .handler(handler_fn(
        |controller: Arc<UsersController>, mut args: HandlerArguments| async move {
            let id = args.take::<UserId>(0)?;
            controller.find_one(id).await.into_framework_response()
        },
    ))
```

The implementation may refine builder names, but it must retain a single erased runtime handler shape.

## Request context

```rust
pub struct RequestContext {
    pub request: FrameworkRequest,
    pub extensions: Extensions,
    pub route: Arc<RouteMetadata>,
    pub controller: Arc<ControllerMetadata>,
    pub container: RequestResolver,
}
```

The context owns or shares request data rather than borrowing the Axum request. This avoids self-referential futures and allows the context to cross middleware and interceptor boundaries. Platform adapters convert native requests at the edge.

Only HTTP transport is represented in 0.1. A transport enum will be introduced when a second transport is implemented, rather than adding uninhabited abstractions now.

## Parameter extraction

```rust
pub trait ParameterExtractor: Send + Sync + 'static {
    fn extract<'a>(
        &'a self,
        context: &'a mut RequestContext,
    ) -> ExtractFuture<'a>;
}

pub type ExtractedValue = Box<dyn Any + Send>;
```

Initial extractors:

- Path value by name.
- Query object or named query value.
- JSON body.
- Header by name.
- Request extension by concrete type.

Body-consuming extractors share a cached body representation in `RequestContext`. More than one body consumer is rejected during route compilation unless the extractors explicitly share the same decoded value.

Extractors return structured `ExtractionError` values containing location, field name, expected type name, and a safe message. Parsing traits are explicit; the framework does not guess string conversions beyond registered extractor behavior.

Validation is a separate parameter stage applied after extraction and before handler invocation. It may replace the extracted value with a transformed value of the declared type.

## Paths

- Controller and route paths are joined at compilation.
- The canonical separator is `/`.
- Empty controller or route paths are allowed and normalize to `/` when combined.
- Duplicate separators are normalized.
- Parameter syntax in the framework API is `:name`; adapters translate it to native syntax when needed.
- Wildcards are deferred.
- Duplicate method plus normalized full path is a startup error.

## Responses

Handlers return values implementing `IntoFrameworkResponse`. The core implementations cover:

- `FrameworkResponse`.
- `Result<T, E>` where `T` and `E` implement response conversion.
- JSON response wrapper.
- Empty response.
- Common text and byte responses.

Framework responses own status, headers, and a transport-neutral body. Streaming body support is deferred but the body enum must be non-exhaustive.

## Error behavior

- Extraction and validation failures are client rejections.
- User handler errors convert through the application's error-response contract.
- Controller/argument downcast failures are internal definition errors.
- Unknown response conversion failures become structured internal errors.
- The adapter must not expose raw type-erasure or parsing details to clients.

Initial codes include `RF_ROUTE_DUPLICATE`, `RF_ROUTE_INVALID_PATH`, `RF_ROUTE_INVALID_BODY_EXTRACTORS`, `RF_HTTP_EXTRACTION_FAILED`, and `RF_HTTP_HANDLER_TYPE_MISMATCH`.

## Axum escape hatch

Transport-neutral route handlers cannot directly accept arbitrary Axum extractors. Applications needing them attach a native route through the Axum adapter escape hatch. This boundary is deliberate: native handlers do not participate automatically in framework guards, interceptors, or route metadata.

## Alternatives considered

- Generating one Axum handler per route was rejected because it couples macros and core metadata to Axum.
- Borrowed execution contexts were rejected because boxed async handler lifetimes become unnecessarily restrictive.
- Serializing extracted values through JSON was rejected because it loses types and adds hot-path overhead.

## Performance impact

Each request performs one controller `Arc` clone, one erased handler dispatch, and one extraction dispatch per parameter. Downcasts are constant time. Benchmarks must compare this path with equivalent Axum handlers.

## Testing strategy

- Definition and path normalization tests.
- Duplicate route and multiple-body-consumer failures.
- Extraction tests for every source and malformed input.
- Handler and response conversion tests.
- Explicit-versus-generated definition parity tests.
- End-to-end Axum adapter tests.

## Unresolved questions

The exact owned body representation and size limit belong to the HTTP implementation. They must allow early rejection and avoid multiple raw-body reads.
