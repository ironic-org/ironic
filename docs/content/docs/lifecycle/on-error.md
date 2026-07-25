---
title: OnError
description: Centralized error handling — log errors, increment metrics, trigger alerts
---

# OnError

`OnError` fires when an **unhandled error** escapes from a controller
or service. It receives the error and the request context.

## Position

```
Controller/Service
  │
  ├── Success → return response
  │
  └── Error ──▶ OnError ◀── YOU ARE HERE
                  │
                  ▼
              Error response sent
```

## Trait

```rust
pub trait OnError {
    async fn on_error(&self, error: &HttpError, context: &RequestContext);
}
```

Note: This hook **cannot fail**. It's fire-and-forget for observability.

## Basic Usage

```rust
pub struct ErrorReporter;

impl OnError for ErrorReporter {
    async fn on_error(&self, error: &HttpError, context: &RequestContext) {
        tracing::error!(
            error = %error,
            status = %error.status_code(),
            method = %context.request().method(),
            path = %context.request().uri(),
            "request failed"
        );

        metrics::counter!("http_errors_total", 1,
            "status" => error.status_code().to_string()
        );
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_error = [ErrorReporter],
)]
pub struct AppModule;
```

## Multiple Handlers

Multiple `OnError` handlers can be registered. All run independently:
```rust
// Both run on every error
lifecycle_error = [ErrorReporter, MetricsRecorder],
```

## Common Uses

- Structured error logging with request context
- Error rate metrics (Prometheus counters)
- Error alerting (PagerDuty, Sentry integration)
- Audit log for security-relevant errors
- Debug header injection for development
