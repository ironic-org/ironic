---
title: TestApplication & Assertions
description: TestApplication/TestModule builder APIs, fluent HTTP request builders, and the TestResponse assertion API.
---

# TestApplication & Assertions

## What you'll learn

- Assemble test apps with the builder API
- Send requests with fluent HTTP method builders
- Assert on responses with `assert_status`, `assert_json`, `assert_error`
- Extract values (status, JSON body, headers) for custom assertions

---

## TestApplication builder API

For advanced scenarios, use the builder to assemble test apps with precise control:

```rust
use ironic::{TestApplication, TestApplicationBuilder};

let app = TestApplication::builder()
    .module::<AppModule>()                 // Register a module by type
    .module::<AuthModule>()                // Add another module
    .override_provider::<UserService>(mock_service)  // Swap a dependency
    .override_provider::<CacheService>(mock_cache)   // Swap another
    .build()                               // Finalize and start
    .await
    .unwrap();

app.shutdown().await.unwrap();
```

| Method | Purpose |
|--------|---------|
| `.module::<T>()` | Register a module (call once per module) |
| `.override_provider::<T>(val)` | Replace a provider of type `T` with `val` |
| `.build()` | Finalize the container and start the application |
| `.shutdown()` | Tear down the container gracefully |

## Request builder methods

The test app provides fluent HTTP method builders:

```rust
// GET — simple path-based request
app.get("/users").send().await;

// POST — with JSON body
app.post("/users")
    .json(&CreateUserDto { name: "Bob".into() })
    .header("Authorization", "Bearer token-abc")
    .send()
    .await;

// PUT — update resource
app.put("/users/1")
    .json(&UpdateUserDto { name: Some("Updated".into()), ..Default::default() })
    .send()
    .await;

// DELETE — remove resource
app.delete("/users/1").send().await;
```

| Method | App Method | Builder Methods |
|--------|-----------|-----------------|
| GET | `app.get(uri)` | `.header(key, val)`, `.send()` |
| POST | `app.post(uri)` | `.json(payload)`, `.header(key, val)`, `.send()` |
| PUT | `app.put(uri)` | `.json(payload)`, `.header(key, val)`, `.send()` |
| DELETE | `app.delete(uri)` | `.header(key, val)`, `.send()` |

> `.header()` can be chained multiple times to set several headers before calling `.send()`.

## TestResponse assertion API

Every `.send()` call returns a `TestResponse` with these methods:

```rust
let resp = app.get("/users/1").send().await;

// Assertions (panic on failure)
resp.assert_status(200);
resp.assert_json(&expected);
resp.assert_error("USER_NOT_FOUND");

// Extraction (returns values)
let status: u16 = resp.status();
let body: serde_json::Value = resp.json();
let headers: HeaderMap = resp.headers();
```

| Method | Returns | Behavior |
|--------|---------|----------|
| `.assert_status(code)` | `()` | Panics if status != code |
| `.assert_json(&T)` | `()` | Panics if body doesn't match |
| `.assert_error(code)` | `()` | Panics if error code doesn't match |
| `.status()` | `u16` | Returns the HTTP status code |
| `.json()` | `serde_json::Value` / `T` | Deserializes the response body |
| `.headers()` | `HeaderMap` | Returns all response headers |

## Fluent assertion API

```rust
// Status code
app.get("/health").send().await.assert_status(200);
app.get("/missing").send().await.assert_status(404);

// JSON body
app.get("/users/1").send().await.assert_json(&UserView {
    id: 1,
    name: "Alice".into(),
});

// Error code
app.get("/users/999").send().await.assert_error("USER_NOT_FOUND");

// Extract raw JSON
let body: serde_json::Value = app.get("/items").send().await.json();
assert_eq!(body.as_array().unwrap().len(), 3);
```

## Testing POST/PUT/DELETE

```rust
// POST with JSON body
app.post("/users")
    .json(&CreateUserDto {
        name: "Bob".into(),
        email: "bob@example.com".into(),
    })
    .send()
    .await
    .assert_status(201);

// PUT with body
app.put("/users/1")
    .json(&UpdateUserDto {
        name: Some("Bob Updated".into()),
        ..Default::default()
    })
    .send()
    .await
    .assert_status(200);

// DELETE
app.delete("/users/1").send().await.assert_status(204);
```

## Try it yourself

1. Write a test that creates a user and verifies it exists
2. Write a test that sends invalid data and checks for a 400 error
3. Verify all three tests pass with `ironic test`

## What you learned

- [x] Builder API: `.module::<T>()`, `.override_provider::<T>(val)`, `.build()`, `.shutdown()`
- [x] Request builders: `.get()`, `.post()`, `.put()`, `.delete()`, `.json()`, `.header()`, `.send()`
- [x] Fluent assertions: `.assert_status()`, `.assert_json()`, `.assert_error()`
