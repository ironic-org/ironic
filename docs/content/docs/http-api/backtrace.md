---
title: Error Backtraces
description: Attach stack backtraces to internal errors for faster debugging — without exposing internals in production.
---

# Error Backtraces

## Enabling

```toml
ironic = { features = ["backtrace"] }
```

## What it does

The `backtrace` feature captures a `std::backtrace::Backtrace` on every `HttpError::internal(...)` call. The backtrace is stored in the error but only serialized into the HTTP response in debug builds.

```rust
use ironic::HttpError;

fn handler() -> Result<(), HttpError> {
    Err(HttpError::internal("DB_ERROR", "connection failed"))
    // With `backtrace` enabled: captures a full stack trace
    // In debug builds:      serialized in the JSON response
    // In release builds:    omitted — only code + message returned
}
```

## Attaching backtraces to existing errors

```rust
use ironic::HttpError;

fn handler() -> Result<(), HttpError> {
    let error = HttpError::bad_request("INVALID_INPUT", "bad value");
    error.with_backtrace() // captures trace at this call site
}
```

## When to use

- **Development**: enables instant debugging from your API responses
- **Staging**: diagnose errors without attaching a debugger
- **Production**: backtraces are stripped from responses; enable this flag to get backtraces in server-side logs without leaking internals

## Feature flags

| Flag | Enables |
|------|---------|
| `backtrace` | `HttpError.with_backtrace()`, auto-capture on `HttpError::internal()` |

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Backtrace not captured | Use `HttpError::internal(...)` or `.with_backtrace()` — other constructors don't capture |
| Backtrace visible in production | Only serialized in `debug_assertions` builds; release builds omit it |
