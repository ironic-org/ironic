---
title: OnError
description: Global error hook — centralized logging, Sentry/DataDog reporting, alerting on specific error codes across the entire application.
---

# OnError

Called on **every unhandled error** before exception filters run. This is the centralized error dispatch point for the entire application.

## When it fires

```
Handler / Middleware / Guard / Pipe returns Err(...)
    │
    ▼
OnError  ← YOU ARE HERE
    │
    ▼
Exception filters (route → controller → global)
    │
    ▼
Default error handler (500)
```

Runs BEFORE any exception filter has a chance to catch the error. This means you always see the raw error, even if a filter later transforms it into a clean 404 response.

## The trait

```rust
pub trait OnError: Send + Sync + 'static {
    fn on_error(&self, error_code: &str, error_message: &str) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnError |
|---|---|
| Send all 5xx errors to Sentry/DataDog | Centralized error reporting |
| Log all 4xx errors for analytics | Don't need per-controller logging |
| Alert on specific error codes (DB_ERROR, REDIS_TIMEOUT) | Operational alerting |
| Increment error counters per code | Prometheus-style metrics |

## Example — Sentry integration

```rust
#[derive(Injectable)]
pub struct ErrorReporter {
    sentry: Arc<SentryClient>,
}

impl OnError for ErrorReporter {
    fn on_error(&self, error_code: &str, error_message: &str) -> LifecycleFuture<'_> {
        let code = error_code.to_owned();
        let msg = error_message.to_owned();
        let sentry = Arc::clone(&self.sentry);
        Box::pin(async move {
            if code.starts_with("DB_") {
                sentry.capture_message(&msg, SentryLevel::Error).await;
            }
            Ok(())
        })
    }
}
```

## Example — metric counter

```rust
#[derive(Injectable)]
pub struct ErrorMetrics {
    counter: Arc<MetricsRegistry>,
}

impl OnError for ErrorMetrics {
    fn on_error(&self, error_code: &str, _error_message: &str) -> LifecycleFuture<'_> {
        let code = error_code.to_owned();
        let counter = Arc::clone(&self.counter);
        Box::pin(async move {
            counter.increment(&format!("ironic.errors.{code}"));
            Ok(())
        })
    }
}
```

## OnError vs ExceptionFilter

| | OnError | ExceptionFilter |
|---|---|---|
| Purpose | Observe/report errors | Transform errors into responses |
| Return type | `Result<(), LifecycleError>` | `Result<FrameworkResponse, HttpError>` |
| Can change response? | No | Yes |
| Runs before filters? | Yes | After |
| Best for | Logging, metrics, alerting | 404 → nice JSON, 500 → safe message |
