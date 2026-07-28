---
title: Validation & Pipes
description: Complete guide to request validation with garde — DTO rules, auto-validation via `#[routes]`, custom validators, and error handling.
---

# Validation & Pipes

## What you'll learn

- Add validation rules to DTOs using `#[garde]` attributes
- `#[routes]` macro auto-validates `#[body]`, `#[form]`, and `#[query]` parameters
- Write custom validation rules
- Handle validation errors with proper HTTP responses
- Validate path params, query params, and headers too

---

## Quick Reference

```toml
# Cargo.toml
ironic = { features = ["validation"] }
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

## How Auto-Validation Works

When the `validation` feature is enabled, the `#[routes]` macro automatically inserts a [`validate_for::<T>()`](#validate_for-api) pipe for every `#[body]`, `#[form]`, and `#[query]` parameter. You don't need to add `#[pipe(validate)]` or call `.validate()` manually — it happens after deserialization and before the handler runs.

| Parameter attribute | Auto-validated? | Requires `T: garde::Validate` |
|---------------------|-----------------|------------------------------|
| `#[body]` | Always | Yes (when `validation` feature is on) |
| `#[form]` | Always | Yes (when `validation` feature is on) |
| `#[query]` | Always | Yes (when `validation` feature is on) |
| `#[param]` | No | — |
| `#[header]` | No | — |

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

## Usage

### 1. Define your DTO

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

> Every field must have a rule or `#[garde(skip)]`. Fields without a `#[garde]` attribute are still validated by garde's default rules.

### 2. Use in your controller

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

The `#[routes]` macro generates a `validate_for::<CreateProductDto>()` pipe internally. No manual piping or `.validate()` calls needed.

### 3. Test validation

```bash
curl -X POST http://localhost:3000/products \
  -H "Content-Type: application/json" \
  -d '{"title": "", "price": -1}'

# → 422 Unprocessable Entity
# { "error": "VALIDATION_FAILED", "message": "title: length must be at least 1. price: must be at least 0.01" }
```

### 4. Query & Form validation

Auto-validation works for `#[form]` and `#[query]` too:

```rust
#[routes]
impl ProductsController {
    #[get("/search")]
    async fn search(
        &self,
        #[query] filters: ProductFilterDto,  // ← validated automatically
    ) -> Result<Json<Vec<Product>>, HttpError> {
        Ok(Json(self.service.search(filters)))
    }

    #[post("/import")]
    async fn import(
        &self,
        #[form] batch: ProductBatchDto,  // ← validated automatically
    ) -> Result<Json<ImportReport>, HttpError> {
        Ok(Json(self.service.import(batch)))
    }
}
```

---

## `validate_for()` API

The auto-validation uses the `validate_for::<T>()` function, which creates a typed [`ParameterPipe`](../fundamentals/pipes).

```rust
use ironic::validate_for;

// Creates a pipe that calls garde::Validate::validate on the value
let pipe: Arc<dyn ParameterPipe> = validate_for::<CreateProductDto>();
```

| Feature flag | Behavior |
|-------------|----------|
| `validation` enabled | Calls `garde::Validate::validate(&value)`, returns `422 VALIDATION_FAILED` on failure |
| `validation` disabled | No-op pipe — passes value through unchanged |

You can use `validate_for` manually in hand-written route definitions:

```rust
RouteDefinition::post("/products")
    .parameter_with_pipe(
        JsonBody::<CreateProductDto>::new(),
        validate_for::<CreateProductDto>(),
    )
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
    Ok(())
}
```

Custom validation functions work with auto-validation — just derive `Validate` with `#[garde(context(...))]` and the `#[routes]` macro picks it up automatically.

---

## Validating Path Params & Query Params

Path params and headers aren't auto-validated (they're parsed by `FromStr`), but parsing pipes handle conversion and validation:

```rust
use ironic::ParseIntPipe;

#[routes]
impl Controller {
    #[get("/:id")]
    async fn get(
        &self,
        #[param] #[pipe(ParseIntPipe)] id: u64,
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

Parsing pipes also work on `#[param]`, `#[header]`, and custom decorators.

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

Nested validation applies automatically through `validate_for` — no extra configuration needed.

---

## Validation Error Format

Every validation failure returns:

```json
{
  "error": "VALIDATION_FAILED",
  "message": "title: length must be at least 1. price: must be at least 0.01"
}
```

**Status code:** `422 Unprocessable Entity`

### Custom Error Mapping

Transform validation errors into structured field errors via [`ExceptionFilter`](./exception-filters):

```rust
use ironic::{ExceptionFilter, FilterContext, HttpError, HttpStatus, Response};

struct ValidationErrorFilter;

impl ExceptionFilter for ValidationErrorFilter {
    fn catch(
        &self,
        error: &HttpError,
        _ctx: &FilterContext,
    ) -> Result<Response, HttpError> {
        if error.code() == "VALIDATION_FAILED" {
            Ok(Response::json(
                HttpStatus::UNPROCESSABLE_ENTITY,
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

Register it at the route, controller, or application level:

```rust
// Route-level
#[routes]
impl MyController {
    #[post]
    #[exception_filter(ValidationErrorFilter)]
    async fn create(&self, #[body] dto: MyDto) -> Result<Json<View>, HttpError> {
        // ...
    }
}

// Or application-level in main.rs
let app = CompiledHttpApplication::new(container, routes)
    .exception_filter(Arc::new(ValidationErrorFilter));
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
| DTO doesn't derive `Validate` | Add `#[derive(Validate)]` to the DTO struct |
| `validation` feature not enabled | `ironic = { features = ["validation"] }` in `Cargo.toml` |
| Missing `#[garde]` on every field | Add a rule or `#[garde(skip)]` to every field |
| Forgot `#[garde(dive)]` on nested structs | Nested structs need `dive` to recurse validation |
| Wrong type for `range` | `range` works on numbers; `length` works on strings |
| Using `String` directly as `#[body]` | Wrap in a DTO struct with `#[derive(Validate)]` |

---

## What you learned

- [x] Add `#[garde]` rules to any DTO with `#[derive(Validate)]`
- [x] `#[routes]` macro auto-validates `#[body]`, `#[form]`, and `#[query]` via `validate_for`
- [x] Manual `validate_for::<T>()` for hand-written route definitions
- [x] Custom validators for business logic beyond struct rules
- [x] Parse pipes for path/query params: `ParseIntPipe`, `ParseFloatPipe`, `ParseBoolPipe`
- [x] Nested validation with `#[garde(dive)]`
- [x] Consistent `VALIDATION_FAILED` error at 422
- [x] Custom error mapping via `ExceptionFilter`
