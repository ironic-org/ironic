---
title: Pipes
description: Transform and validate request parameters before they reach your handler.
---

# Pipes

Pipes transform and validate request parameters before they reach your handler. They run after the guard checks and pass the transformed value to the handler.

## How pipes work

```
Guard → Pipe → Handler
         │
         ▼
    Error → 400 response
```

A pipe receives a raw parameter value and returns either a transformed value or an error:

```rust
pub trait Pipe<T, U> {
    type Error: Into<HttpError>;
    fn transform(&self, value: T) -> Result<U, Self::Error>;
}
```

## Built-in pipes

| Pipe | Input | Output | Description |
|------|-------|--------|-------------|
| `ParseIntPipe` | `String` | `i32` | Parses an integer |
| `ParseFloatPipe` | `String` | `f64` | Parses a float |
| `ParseBoolPipe` | `String` | `bool` | Parses a boolean |
| `DefaultValuePipe` | `Option<T>` | `T` | Provides a default if missing |
| `ValidationPipe` | `T` | `T` | Validates with `garde` |
| `UuidPipe` | `String` | `uuid::Uuid` | Parses a UUID |

## Using pipes

```rust
use ironic::*;

#[routes]
impl UserController {
    #[get("/users/{id}")]
    async fn get(
        &self,
        id: PathParameter<ParseIntPipe>,   // validates and parses to i32
    ) -> Json<User> {
        let user_id: i32 = *id;
        // ...
    }

    #[get("/search")]
    async fn search(
        &self,
        query: QueryParameters,
        page: QueryParameter<DefaultValuePipe<ParseIntPipe>>,  // defaults to 0
    ) -> Json<Vec<User>> {
        let page_num: i32 = *page;
        // ...
    }
}
```

## Writing a custom pipe

```rust
struct SlugPipe;

impl Pipe<String, String> for SlugPipe {
    type Error = HttpError;

    fn transform(&self, value: String) -> Result<String, Self::Error> {
        let slug = value.to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "");

        if slug.is_empty() {
            return Err(HttpError::bad_request(
                "INVALID_SLUG",
                "Slug must contain at least one alphanumeric character",
            ));
        }

        Ok(slug)
    }
}
```

## Pipe chaining

Pipes can be chained for combined transformation and validation:

```rust
// Parses integer, then validates it's positive
#[get("/items/{id}")]
async fn get(&self, id: PathParameter<ValidationPipe<ParseIntPipe>>) -> Json<Item> {
    // ...
}
```

## Validation pipe

The `ValidationPipe` integrates with the `garde` validation library:

```rust
use garde::Validate;

#[derive(Debug, Deserialize, Validate)]
struct CreateUser {
    #[garde(length(min = 3, max = 50))]
    name: String,

    #[garde(email)]
    email: String,

    #[garde(range(min = 18))]
    age: u32,
}

#[routes]
impl UserController {
    #[post("/users")]
    async fn create(
        &self,
        body: JsonBody<ValidationPipe<CreateUser>>,
    ) -> Json<User> {
        let validated: CreateUser = body.0;
        // name, email, age are all validated
    }
}
```

## Test pipe

```rust
#[test]
fn test_slug_pipe() {
    let pipe = SlugPipe;
    assert_eq!(pipe.transform("Hello World".into()).unwrap(), "hello-world");
    assert_eq!(pipe.transform("  Foo  Bar  ".into()).unwrap(), "foo-bar");
    assert!(pipe.transform("!!!".into()).is_err());
}
```
