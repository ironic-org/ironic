---
title: OnError
description: Centralized error handling — called on every unhandled error before exception filters.
---

# OnError

Called on every **unhandled error** before exception filters run. This is the centralized place to log errors, report to external monitoring, and track error rates.

## Use cases

- Centralized error logging (with full context)
- Reporting to Sentry, DataDog, or other APM tools
- Alerting on specific error codes
- Tracking error metrics and rates
- Error auditing for compliance

## Signature

```rust
pub trait OnError: Send + Sync + 'static {
    fn on_error(&self, error_code: &str, error_message: &str) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnError, LifecycleError};

struct ErrorReporter {
    sentry_dsn: String,
}

impl OnError for ErrorReporter {
    async fn on_error(&self, error_code: &str, error_message: &str) -> Result<(), LifecycleError> {
        tracing::error!(error_code, error_message, "Unhandled error");
        send_to_sentry(&self.sentry_dsn, error_code, error_message).await;
        metrics::counter!("app.errors.total", "code" => error_code.to_string()).increment(1);
        Ok(())
    }
}
```

## When it runs

```
Handler error
    |
    v
OnError  -->  Exception Filter  -->  Response
```

`OnError` runs **before** exception filters, so you can log or report before the filter transforms the error into a response.

## Registration

```rust
ModuleDefinition::builder::<ErrorReporter>()
    .on_error()
    .build()
```

## Common patterns

### Reporting to Sentry

```rust
impl OnError for SentryReporter {
    async fn on_error(&self, code: &str, msg: &str) -> Result<(), LifecycleError> {
        sentry::capture_message(msg, sentry::Level::Error);
        Ok(())
    }
}
```

### Error rate alerting

```rust
impl OnError for MetricsReporter {
    async fn on_error(&self, code: &str, _msg: &str) -> Result<(), LifecycleError> {
        if code.contains("_NOT_FOUND") || code.contains("_DENIED") {
            return Ok(()); // skip expected errors
        }
        self.alert_if_high_rate(code).await;
        Ok(())
    }
}
```

## Best practices

- Keep `OnError` fast — it runs synchronously before the error response is sent
- Don't re-throw or modify the error — use exception filters for that
- Use error codes to categorize errors, not raw messages
- Filter out expected errors to avoid noise
