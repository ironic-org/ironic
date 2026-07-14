---
title: Validation & Pipes
description: Validate request data automatically with garde — catch bad input before it reaches your business logic.
---

# Validation & Pipes

## What you'll learn

- Add `#[garde]` validation rules to your DTOs
- Use `ValidationPipe` to enforce rules automatically
- Return helpful error messages when validation fails
- Validate numbers, strings, and custom rules

## The big picture

Bad data should never reach your business logic. Validation catches it **at the door**:

```
Request ──► ValidationPipe ──► ❌ "title is too short" (400 Bad Request)
                  │
                  ▼ (valid)
              Controller → Service → ✅ Success
```

## Step 1: Enable validation

```toml
# Cargo.toml
ironic = { features = ["validation"] }
garde = "0.22"
```

## Step 2: Add validation rules

Use `#[derive(Validate)]` and `#[garde(...)]` attributes:

```rust
use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[garde(length(min = 2, max = 100))]
    pub name: String,                    // ← Must be 2-100 characters

    #[garde(email)]
    pub email: String,                   // ← Must be a valid email

    #[garde(range(min = 13, max = 150))]
    pub age: u8,                         // ← Must be 13-150

    #[garde(skip)]                       // ← Don't validate this field
    pub notes: Option<String>,
}
```

## Step 3: Apply ValidationPipe

```rust
use ironic::{ValidationPipe, JsonBody};
use std::sync::Arc;

// In your controller's route definition:
let route = RouteDefinition::new(
    HttpMethod::POST, "/", "create_user",
    handler_fn(|_c: Arc<UserController>, mut args| async move {
        let input = args.take::<CreateUserDto>(0)?;
        // input is guaranteed valid here!
        Ok(Json(format!("Created user {}", input.name)))
    }),
)
.unwrap()
.parameter_with_pipe(
    JsonBody::<CreateUserDto>::new(),
    Arc::new(ValidationPipe),   // ← Validates before handler runs
);
```

## Built-in validation rules

| Rule | What it checks | Example |
|------|---------------|---------|
| `#[garde(length(min = 1, max = 100))]` | String length | Title must be 1-100 chars |
| `#[garde(range(min = 0, max = 100))]` | Number range | Score must be 0-100 |
| `#[garde(email)]` | Valid email format | `user@domain.com` |
| `#[garde(url)]` | Valid URL format | `https://example.com` |
| `#[garde(pattern("^[a-z]+$"))]` | Regex match | Only lowercase letters |
| `#[garde(required)]` | Option must be Some | Field cannot be None |
| `#[garde(skip)]` | Don't validate | Skip this field |

## What happens when validation fails?

The client gets a clear error response:

```json
// POST /users with { "name": "A", "age": 200 }
{
  "error": "VALIDATION_FAILED",
  "message": "Validation failed: name: length must be at least 2. age: must be between 13 and 150"
}
```

HTTP status code: **400 Bad Request**

## Try it yourself

1. Create a `CreateProductDto` with title (1-256 chars) and price (0.01-99999.99)
2. Apply `ValidationPipe` to a POST route
3. Test with invalid data: empty title, negative price
4. Verify you get 400 errors with clear messages

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Forgot `#[garde(...)]` on a field | The field won't be validated! Every field needs a rule or `#[garde(skip)]` |
| `garde` not in Cargo.toml | Add both `ironic = { features = ["validation"] }` AND `garde = "0.22"` |
| Wrong type for range | `#[garde(range(min = 0))]` works on numbers, not strings |
| Validation not applied | Make sure `ValidationPipe` is added to the route with `.parameter_with_pipe()` |

## What you learned

- [x] Add validation rules with `#[derive(Validate)]` and `#[garde(...)]`
- [x] Apply `ValidationPipe` to enforce rules
- [x] Return 400 errors with helpful messages
- [x] Skip optional fields with `#[garde(skip)]`
