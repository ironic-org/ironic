---
title: UUID Parameter Parsing
description: Parse UUID path and query parameters with automatic validation.
---

# UUID Parameter Parsing

## Enabling

```toml
ironic = { features = ["uuid"] }
```

## ParseUUIDPipe

Validates and converts a string parameter into a `uuid::Uuid`. Returns 400 when the value isn't a valid UUID.

```rust
use ironic::{ParseUUIDPipe, parse_uuid, HttpError};
use uuid::Uuid;

// As a route-level pipe:
#[get("/users/:id")]
async fn get_user(
    // ParseUUIDPipe is the pipe; the parameter type is inferred
    #[param(pipe = ParseUUIDPipe, name = "id")]
    id: Uuid,
) -> Result<String, HttpError> {
    Ok(format!("user {id}"))
}
```

## Using the shared constructor

```rust
use ironic::parse_uuid;

// Creates a shared Arc<dyn ParameterPipe>
let pipe = parse_uuid();
```

## Feature flags

| Flag | Enables |
|------|---------|
| `uuid` | `ParseUUIDPipe`, `parse_uuid()` constructor |

## Common mistakes

| Mistake | Fix |
|---------|-----|
| UUID validation fails unexpectedly | Ensure the string is in standard format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` |
| Forgot to enable `uuid` feature | `ironic = { features = ["uuid"] }` — `ParseUUIDPipe` won't compile without it |
