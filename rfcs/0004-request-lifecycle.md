# RFC 0004: Request Lifecycle

- Status: Accepted for initial implementation
- Target: RustFrame 0.1

## Summary

RustFrame executes requests through a deterministic nested pipeline: framework middleware, guards, interceptor pre-processing, extraction and validation, handler execution, interceptor post-processing, then error-to-response mapping. Platform middleware remains outside this framework pipeline.

## Normative order

```text
native platform middleware
  framework middleware (outermost to innermost)
    guards (in registration order)
      interceptors before (outermost to innermost)
        parameter extraction (declaration order)
        parameter transformation/validation (declaration order)
        controller handler
      interceptors after (innermost to outermost)
  framework error mapping
native platform response middleware
```

Global components run before controller components, which run before route components. Within each level, declaration order is preserved. Interceptors nest, so their completion order is reversed.

## Middleware

```rust
pub trait Middleware: Send + Sync {
    fn handle<'a>(
        &'a self,
        context: &'a mut RequestContext,
        next: MiddlewareNext<'a>,
    ) -> ResponseFuture<'a>;
}
```

Middleware can short-circuit with a response or error. A middleware that does not call `next` prevents guards and all later stages from running. Framework middleware operates on transport-neutral request state. Tower middleware is registered separately on the Axum adapter.

## Guards

Guards return `Allow`, `Deny`, or a structured error. `Deny` maps to a configurable forbidden response; authentication guards should return an explicit unauthorized error when appropriate. Evaluation stops at the first non-allow result.

Guards cannot wrap handler completion. They may attach typed values, such as an authenticated principal, to request extensions.

## Interceptors

Interceptors receive a `CallNext` value and may:

- Inspect request and route metadata before calling the next stage.
- Short-circuit with a response or error.
- Transform a successful response after awaiting the next stage.
- Observe or transform structured errors before global error mapping.

An interceptor must call `next` at most once. The API consumes `CallNext` to enforce this rule.

## Extraction and validation

Parameters are extracted and validated in method declaration order for deterministic errors and body handling. The first failure stops processing. A future optimization may parallelize explicitly independent extractors, but 0.1 does not.

Validation failures use status 422 by default. Syntax or decoding failures use status 400. Applications can replace the error mapper but not the pipeline order.

## Errors

Every stage returns `Result<FrameworkResponse, FrameworkError>`. User error types are converted into a framework error or response at the handler adapter boundary.

Error mapping occurs once, after framework middleware unwinds. Consequently:

- Interceptors can observe errors from inner stages.
- Middleware can observe errors if it awaits `next`.
- The global mapper receives an error only if no earlier component converted it to a response.
- Error mapper failures fall back to a minimal redacted 500 response and are logged.

Errors carry a stable public code, safe message, optional status, and internal source/context. Internal sources are never serialized by default.

## Panic boundary

RustFrame 0.1 does not promise recovery from process-wide aborts. When panic unwinding is enabled, the Axum adapter installs a per-request panic-catching boundary around the framework pipeline and maps a panic to a redacted 500 response while logging request and route identifiers. Applications must not rely on this for memory or invariant safety.

## Registration snapshots

Pipeline component lists are compiled at application startup into immutable `Arc` slices. Requests do not acquire mutation locks and a running application cannot add global components.

## Cancellation

Dropping the platform request future cancels the framework pipeline. Components that spawn background work own its cancellation semantics. RustFrame does not detach handler futures automatically. Shutdown stops accepting new requests and allows the adapter's configured graceful-shutdown deadline to govern in-flight work.

## Error and ordering tests

The test suite must record exact stage entry and exit for:

- Success.
- Middleware short-circuit.
- Guard denial and guard error.
- Extraction and validation failure.
- Handler error.
- Interceptor short-circuit and error transformation.
- Middleware error transformation.
- Panic with unwinding enabled.

## Alternatives considered

- Extracting before guards was rejected because unauthorized requests should not deserialize potentially large or sensitive bodies.
- Error mapping inside each stage was rejected because it prevents outer components from observing consistent errors.
- Concurrent guard or parameter execution was rejected because deterministic ordering and extension mutation are more valuable for 0.1.

## Performance impact

The compiled pipeline requires one erased call per active component and no per-request registration locks. Empty component lists should collapse to direct next-stage calls. Benchmarks will cover no-op and representative pipelines.

## Unresolved questions

Graceful-shutdown timeout defaults and logging fields are operational configuration decisions for Phase 10, not pipeline-contract blockers.
