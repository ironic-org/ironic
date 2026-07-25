---
title: OnGuardDenied
description: Track authentication and authorization failures — audit logging, suspicious activity detection
---

# OnGuardDenied

`OnGuardDenied` fires when a **Guard** returns `GuardDecision::Deny`,
rejecting access to a route or controller.

## Position

```
Incoming Request
  │
  ▼
Guard.check()
  │
  ├── Allow → continue to handler
  │
  └── Deny ──▶ OnGuardDenied ◀── YOU ARE HERE
                  │
                  ▼
              403 Forbidden response
```

## Trait

```rust
pub trait OnGuardDenied {
    async fn on_guard_denied(&self, context: &RequestContext, guard: &str);
}
```

## Basic Usage

```rust
pub struct AuditLogger;

impl OnGuardDenied for AuditLogger {
    async fn on_guard_denied(&self, context: &RequestContext, guard: &str) {
        tracing::warn!(
            guard = %guard,
            path = %context.request().uri(),
            method = %context.request().method(),
            "access denied — guard rejected request"
        );

        metrics::counter!("auth_denied_total", 1,
            "guard" => guard,
        );
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_guard_denied = [AuditLogger],
)]
pub struct AppModule;
```

## Common Uses

- Audit logging for security compliance
- Rate limit tracking (too many denied requests)
- Suspicious activity detection (brute force attempts)
- Alerting on repeated denial patterns
