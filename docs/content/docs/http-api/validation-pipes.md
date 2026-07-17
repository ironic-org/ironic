---
title: Validation & Pipes
description: Complete guide to request validation with garde — DTO rules, custom validators, macro-based controllers, and error handling.
---

# Validation & Pipes

## What you'll learn

- Add validation rules to DTOs using `#[garde]` attributes
- Apply `ValidationPipe` via attribute-based controllers, route-level pipes, and controller-level pipes
- Write custom validation rules
- Handle validation errors with proper HTTP responses
- Validate path params, query params, and headers too

---

## Quick Reference

```toml
# Cargo.toml
ironic = { features = ["validation"] }
garde = "0.23"
```

```rust
use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[garde(length(min = 2, max = 100))]
    pub name: String,
    #[garde(email)]
    pub email: String,
    #[garde(range(min = 13, max = 150))]
    pub age: u8,
    #[garde(skip)]
    pub bio: Option<String>,
}
```

---

## All garde Validation Rules

### String Rules

| Rule | Example | What it checks |
|------|---------|---------------|
| `length(min, max)` | `#[garde(length(min = 1, max = 256))]` | String character count |
| `email` | `#[garde(email)]` | Valid email format |
| `url` | `#[garde(url)]` | Valid URL format |
| `pattern(regex)` | `#[garde(pattern("^[a-z0-9_]+$"))]` | Regex match |
| `contains(substring)` | `#[garde(contains("@"))]` | Substring present |
| `prefix(prefix)` | `#[garde(prefix("https://"))]` | Starts with |
| `suffix(suffix)` | `#[garde(suffix(".com"))]` | Ends with |
| `ascii` | `#[garde(ascii)]` | Only ASCII characters |
| `alphanumeric` | `#[garde(alphanumeric)]` | Only letters and digits |

### Number Rules

| Rule | Example | What it checks |
|------|---------|---------------|
| `range(min, max)` | `#[garde(range(min = 0, max = 150))]` | Integer/float bounds |
| `greater_than(val)` | `#[garde(greater_than(0))]` | Must exceed value |
| `less_than(val)` | `#[garde(less_than(100))]` | Must be below value |
| `positive` | `#[garde(positive)]` | Must be > 0 |

### General Rules

| Rule | When to use |
|------|------------|
| `required` | Ensure `Option<T>` is `Some` |
| `skip` | Skip validation for this field |
| `dive` | Validate nested structs |
| `custom(fn)` | Custom validation function |

---

## Approach 1: Macro-Based Controllers

The simplest approach — just derive `Validate` and the framework handles it.

### Step 1: Define your DTO

```rust
use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductDto {
    #[garde(length(min = 1, max = 256))]
    pub title: String,
    #[garde(range(min = 0.01))]
    pub price: f64,
    #[garde(range(min = 0))]
    pub stock: u32,
    #[garde(skip)]
    pub tags: Option<Vec<String>>,
}
```

### Step 2: Use in your controller

```rust
#[controller("/products")]
#[derive(Injectable)]
pub struct ProductsController {
    service: Arc<ProductsService>,
}

#[routes]
impl ProductsController {
    #[post]
    async fn create(
        &self,
        #[body] dto: CreateProductDto,  // ← validated automatically
    ) -> Result<Json<Product>, HttpError> {
        Ok(Json(self.service.create(dto)))
    }
}
```

> The `#[body]` extractor auto-applies `ValidationPipe` when the `validation` feature is enabled and the DTO derives `Validate`.

### Step 3: Test validation

```bash
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -d '{"title": "", "price": -1}'

# → 400 Bad Request
# { "error": "VALIDATION_FAILED", "message": "title: length must be at least 1. price: must be at least 0.01" }
```

---

## Approach 2: Route-Level Pipe

Attach `ValidationPipe` to a specific route handler's parameter:

```rust
use ironic::prelude::*;
use std::sync::Arc;

#[controller("/products")]
#[derive(Injectable)]
pub struct ProductsController {
    service: Arc<ProductsService>,
}

#[routes]
impl ProductsController {
    #[post("/")]
    async fn create(
        &self,
        #[body] #[pipe(ValidationPipe)] dto: CreateProductDto,
    ) -> Result<Json<Product>, HttpError> {
        Ok(Json(self.service.create(dto)))
    }
}
```

## Approach 3: Controller-Level Pipe

Define the controller with `#[controller]` and register the pipe via builder:

```rust
// Controller definition
#[controller("/products")]
#[derive(Injectable)]
pub struct ProductsController {
    service: Arc<ProductsService>,
}

#[routes]
impl ProductsController {
    #[post("/")]
    async fn create(
        &self,
        #[body] dto: CreateProductDto,
    ) -> Result<Json<Product>, HttpError> {
        Ok(Json(self.service.create(dto)))
    }
}

// Apply pipe to every route when registering the module:
use ironic::{Module, ControllerDefinition};
use std::sync::Arc;

#[derive(Module)]
#[module(custom_setup)]
struct ProductsModule;

impl ProductsModule {
    fn custom_setup(definition: ModuleDefinition) -> ModuleDefinition {
        definition.override_controller::<ProductsController>(
            ControllerDefinition::from_type::<ProductsController>()
                .pipe(Arc::new(ValidationPipe)),
        )
    }
}
```

## Approach 4: Application-Level Pipe

Apply to ALL routes everywhere:

```rust
use ironic::CompiledHttpApplication;

let app = CompiledHttpApplication::new(container, routes)
    .pipe(Arc::new(ValidationPipe));
```

---

## Custom Validation Functions

For business-logic validation beyond struct-level rules:

```rust
use garde::Validate;
use ironic::HttpError;

#[derive(Debug, Deserialize, Validate)]
#[garde(context(PasswordContext))]
pub struct CreateUserDto {
    #[garde(length(min = 2, max = 100))]
    pub name: String,
    #[garde(custom(validate_password))]
    pub password: String,
    #[garde(custom(validate_password_confirmation))]
    pub password_confirmation: String,
}

// Context for custom validation
struct PasswordContext {
    min_length: usize,
}

fn validate_password(value: &str, ctx: &PasswordContext) -> garde::Result {
    if value.len() < ctx.min_length {
        return Err(garde::Error::new("password too short"));
    }
    if !value.chars().any(|c| c.is_uppercase()) {
        return Err(garde::Error::new("password must contain an uppercase letter"));
    }
    if !value.chars().any(|c| c.is_numeric()) {
        return Err(garde::Error::new("password must contain a number"));
    }
    Ok(())
}

fn validate_password_confirmation(
    value: &str,
    ctx: &PasswordContext,
) -> garde::Result {
    // Compare with password field — see garde docs for field comparison
    Ok(())
}
```

---

## Validating Path Params & Query Params

Validation works everywhere — not just bodies:

```rust
use ironic::ParseIntPipe;

#[routes]
impl Controller {
    #[get("/:id")]
    async fn get(
        &self,
        #[param] #[pipe(ParseIntPipe)] id: u64,  // ← validates and converts
    ) -> Result<Json<User>, HttpError> {
        // id is guaranteed to be a valid u64
    }

    #[get("/search")]
    async fn search(
        &self,
        #[query] #[pipe(ParseIntPipe)] page: u64,
        #[query] limit: Option<u64>,
    ) -> Result<Json<Vec<User>>, HttpError> {
        // page is guaranteed valid
    }
}
```

### Built-in Parsing Pipes

| Pipe | Converts | Example |
|------|----------|---------|
| `ParseIntPipe` | String → `i64` | `/items/42` |
| `ParseFloatPipe` | String → `f64` | `?price=9.99` |
| `ParseBoolPipe` | String → `bool` | `?active=true` |
| `ParseUUIDPipe` | String → `Uuid` | `/users/uuid-value` |

---

## Nested Object Validation

Validate deep structures with `#[garde(dive)]`:

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderDto {
    #[garde(dive)]
    pub customer: CustomerDto,
    #[garde(length(min = 1))]
    #[garde(dive)]
    pub items: Vec<OrderItemDto>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CustomerDto {
    #[garde(length(min = 2))]
    pub name: String,
    #[garde(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct OrderItemDto {
    #[garde(range(min = 1))]
    pub product_id: u64,
    #[garde(range(min = 1, max = 999))]
    pub quantity: u32,
}
```

---

## Validation Error Format

Every validation failure returns:

```json
{
  "error": "VALIDATION_FAILED",
  "message": "Validation failed: title: length must be at least 1. price: must be at least 0.01"
}
```

**Status code:** `400 Bad Request`

### Custom Error Mapping

Transform validation errors into structured field errors:

```rust
struct ValidationErrorFilter;

impl ExceptionFilter for ValidationErrorFilter {
    fn catch(
        &self,
        error: &HttpError,
        _ctx: &FilterContext,
    ) -> Result<FrameworkResponse, HttpError> {
        if error.code() == "VALIDATION_FAILED" {
            Ok(FrameworkResponse::json(
                HttpStatus::BAD_REQUEST,
                &serde_json::json!({
                    "error": "VALIDATION_FAILED",
                    "fields": {
                        "title": ["length must be at least 1"],
                        "price": ["must be at least 0.01"]
                    }
                }),
            )
            .unwrap())
        } else {
            Err(error.clone())
        }
    }
}
```

---

## Complete Example

```rust
// dto/create_user_dto.rs
use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[garde(length(min = 2, max = 100))]
    pub name: String,
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 8, max = 128))]
    pub password: String,
    #[garde(range(min = 13, max = 150))]
    pub age: u8,
    #[garde(skip)]
    pub bio: Option<String>,
}

// controller/user_controller.rs
#[controller("/users")]
#[derive(Injectable)]
pub struct UserController {
    service: Arc<UserService>,
}

#[routes]
impl UserController {
    #[post]
    async fn create(
        &self,
        #[body] dto: CreateUserDto,  // ← validated automatically
    ) -> Result<Json<UserView>, HttpError> {
        Ok(Json(self.service.create(dto)?.into()))
    }
}
```

---

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| DTO doesn't derive `Validate` | Add `#[derive(Validate)]` |
| Missing `#[garde]` on every field | Add a rule or `#[garde(skip)]` to every field |
| `garde` crate not in Cargo.toml | `cargo add garde` |
| `validation` feature not enabled | `ironic = { features = ["validation"] }` |
| Wrong type for `range` | `range` works on numbers; `length` works on strings |
| Forgot `#[garde(dive)]` on nested structs | Nested structs need `dive` to recurse |

## What you learned

- [x] Add `#[garde]` rules to any DTO with `#[derive(Validate)]`
- [x] Macro-based controllers auto-validate `#[body]` parameters
- [x] Route-level pipe via `#[pipe(ValidationPipe)]` on handler parameters
- [x] Custom validators for business logic beyond struct rules
- [x] Parse pipes for path/query params: `ParseIntPipe`, `ParseFloatPipe`, `ParseBoolPipe`
- [x] Nested validation with `#[garde(dive)]`
- [x] Consistent `VALIDATION_FAILED` error at 400
- [x] Custom error mapping via `ExceptionFilter`
