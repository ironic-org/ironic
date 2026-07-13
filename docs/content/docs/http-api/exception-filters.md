---
title: Exception filters
description: Catch and transform errors at global, controller, and route scope with typed exception filters.
---

# Exception filters

Exception filters let you intercept errors thrown by guards, interceptors, pipes, or handlers and
transform them into controlled responses. Filters are typed and follow scope precedence: route
filters run first, then controller filters, then global filters.

## Defining a filter

Implement the `ExceptionFilter` trait:

```rust
use ironic::{ExceptionFilter, FilterContext, HttpError, FrameworkResponse, HttpStatus};

struct ValidationFilter;

impl ExceptionFilter for ValidationFilter {
    fn catch(&self, error: &HttpError, _ctx: &FilterContext) -> Result<FrameworkResponse, HttpError> {
        if error.code() == "IRONIC_VALIDATION_FAILED" {
            Ok(FrameworkResponse::error(
                HttpStatus::UNPROCESSABLE_ENTITY,
                "VALIDATION",
                error.message(),
            ))
        } else {
            Err(HttpError::bad_request("UNHANDLED", "not handled by this filter"))
        }
    }
}
```

## Filter context

`FilterContext` provides the route metadata that was active when the error occurred:

```rust
use ironic::{ExceptionFilter, FilterContext, HttpError, FrameworkResponse, CacheMetadata};

struct CacheAwareFilter;

impl ExceptionFilter for CacheAwareFilter {
    fn catch(&self, error: &HttpError, ctx: &FilterContext) -> Result<FrameworkResponse, HttpError> {
        if let Some(cache) = ctx.route_metadata().get::<CacheMetadata>() {
            // Custom handling when caching was enabled on this route
        }
        Err(HttpError::bad_request("DEFAULT", "fall through"))
    }
}
```

## Scope registration

```rust
// Global: catches errors from every route
CompiledHttpApplication::new(container, routes)
    .exception_filter(Arc::new(GlobalExceptionFilter));

// Controller: catches errors from all routes in this controller
ControllerDefinition::new::<UsersController>("/users", provider)?
    .exception_filter(Arc::new(ControllerExceptionFilter));

// Route: catches errors only from this specific route
RouteDefinition::new(HttpMethod::GET, "/users", "list", handler_fn(handler))?
    .exception_filter(Arc::new(RouteExceptionFilter));
```

## Default behavior

When no filter matches an error, Ironic wraps the error as a JSON response with the status code,
error code, and safe message from the `HttpError`. The default shape is:

```json
{ "status": 400, "code": "RF_HTTP_...", "message": "..." }
```
