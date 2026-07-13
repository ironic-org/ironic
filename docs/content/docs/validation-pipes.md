---
title: Validation pipes
description: Transform and validate request parameters with typed built-in pipes and custom validation logic.
---

# Validation pipes

Ironic pipes sit between parameter extraction and the handler. Each pipe transforms or validates one
handler parameter and short-circuits the request on failure.

## Built-in parsing pipes

```rust
use ironic::{parse_int, parse_float, parse_bool, parse_uuid};

RouteDefinition::new(HttpMethod::GET, "/user/:id", "get_user", handler_fn(handler))?
    .parameter_with_pipe(PathParameter::<String>::new("id"), parse_int())
    .parameter_with_pipe(QueryParameters::<String>::new(), parse_float())
```

| Pipe | Input | Output | Error code |
|------|-------|--------|------------|
| `parse_int()` | `String` | `i64` | `RF_PARSE_INT_FAILED` |
| `parse_float()` | `String` | `f64` | `RF_PARSE_FLOAT_FAILED` |
| `parse_bool()` | `String` | `bool` | `RF_PARSE_BOOL_FAILED` |
| `parse_uuid()` | `String` | `Uuid` | `RF_PARSE_UUID_FAILED` |

## Validation with `garde`

Enable `validation` to use the [`garde`](https://crates.io/crates/garde) derive macro for declarative
field validation:

```toml
ironic = { features = ["validation"] }
```

```rust
use garde::Validate;
use ironic::{JsonBody, ValidationPipe};

#[derive(Validate)]
struct CreateUser {
    #[garde(length(min = 3, max = 50))]
    name: String,
    #[garde(email)]
    email: String,
    #[garde(range(min = 18, max = 120))]
    age: u8,
}

RouteDefinition::new(HttpMethod::POST, "/users", "create_user", handler_fn(handler))?
    .parameter_with_pipe(JsonBody::<CreateUser>::new(), ValidationPipe::new())
```

`ValidationPipe` calls `garde::Validate::validate` on the extracted value and returns a
`422 Unprocessable Entity` with `IRONIC_VALIDATION_FAILED` on failure.

## Custom pipes

Implement `ParameterPipe` for ad-hoc validation or transformation:

```rust
use ironic::{ParameterPipe, PipeFuture, ExtractedValue, HttpError, RequestContext};

struct TrimPipe;

impl ParameterPipe for TrimPipe {
    fn transform<'a>(&'a self, value: ExtractedValue, _ctx: &'a mut RequestContext) -> PipeFuture<'a> {
        Box::pin(async move {
            let s = value.downcast::<String>().map_err(|_| {
                HttpError::bad_request("TYPE_ERROR", "expected string")
            })?;
            Ok(Box::new(s.trim().to_string()) as ExtractedValue)
        })
    }
    fn description(&self) -> &'static str { "trim" }
}
```

Create a factory for ergonomic chaining:

```rust
use ironic::pipe_fn;

let trim_pipe = pipe_fn::<String, String, _>(|value| Ok(value.trim().to_string()));
```

## Pipe scoping

Pipes can be registered at three levels, applied in order of **global → controller → route**:

```rust
// Global pipe applied to every parameter in the application
CompiledHttpApplication::new(container, routes)
    .pipe(&ValidationPipe::new());

// Controller pipe applied to every route parameter
ControllerDefinition::new::<UsersController>("/users", provider)?
    .pipe(ValidatorPipe::new());

// Route-level pipe applied to this parameter only
RouteDefinition::new(...)?
    .parameter_with_pipe(JsonBody::new(), parse_int());
```
