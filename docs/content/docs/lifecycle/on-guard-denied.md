---
title: OnGuardDenied
description: Hook for centralized auth failure tracking when a Guard denies access.
---

# OnGuardDenied

Called when any `Guard` returns `GuardDecision::Deny`. This is the centralized place to track authentication and authorization failures.

## Use cases

- Centralized auth failure logging
- Brute-force detection (tracking failed logins per IP)
- Rate-limit counters per guard type
- Security auditing
- Alerting on repeated denial patterns

## Signature

```rust
pub trait OnGuardDenied: Send + Sync + 'static {
    fn on_guard_denied(&self, guard_name: &str) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnGuardDenied, LifecycleError};

struct SecurityAuditor;

impl OnGuardDenied for SecurityAuditor {
    async fn on_guard_denied(&self, guard_name: &str) -> Result<(), LifecycleError> {
        tracing::warn!(guard_name, "Access denied");
        metrics::counter!("auth.denials", "guard" => guard_name.to_string())
            .increment(1);
        Ok(())
    }
}
```

## When it runs

```
Guard::allow()
    |
    +-- true  --> continue to handler
    |
    +-- false --> OnGuardDenied --> 403/401 response
```

## Registration

```rust
ModuleDefinition::builder::<SecurityAuditor>()
    .guard_denied()
    .build()
```

## Brute-force detection

```rust
impl OnGuardDenied for RateLimitTracker {
    async fn on_guard_denied(&self, guard_name: &str) -> Result<(), LifecycleError> {
        if guard_name == "login" {
            self.record_failed_attempt().await;
            if self.recent_failures() > 5 {
                alert!("Brute force attack detected");
            }
        }
        Ok(())
    }
}
```

## Best practices

- Use `guard_name` to identify which guard denied — not where it happened
- Track per-IP counters for rate limiting
- Don't block the request — `OnGuardDenied` runs after the denial decision
