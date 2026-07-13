---
title: Task scheduling
description: Schedule recurring tasks with fixed intervals or cron expressions using cooperative scheduling.
---

# Task scheduling

Enable `scheduling` to spawn repeating background tasks with cooperative shutdown. Add `cron`
for cron-expression-based scheduling.

```toml
ironic = { features = ["scheduling"] }
```

## Fixed-interval tasks

```rust
use ironic::services::scheduling::{interval, ScheduledTask};
use std::time::Duration;

let task: ScheduledTask = interval(Duration::from_secs(30), || async move {
    // Runs every 30 seconds
    reconcile_subscriptions().await;
});
```

Missed ticks are skipped when the task runs longer than the interval. The first invocation fires
after one full period, not immediately.

## Cron-expression tasks

Enable `cron` for expressive scheduling:

```toml
ironic = { features = ["scheduling", "cron"] }
```

```rust
use ironic::services::scheduling::cron_schedule;

let task = cron_schedule("0 0 2 * * *", || async move {
    // Runs daily at 2:00 AM
    nightly_report().await;
})?;
```

Supports the standard six-field cron format: `sec min hour day-of-month month day-of-week`.
Invalid expressions return a parse error.

## Shutdown

Tasks are cooperative. Call `shutdown()` to stop a task gracefully after the current invocation
completes. Call `abort()` for immediate termination.

```rust
task.shutdown().await?;
// or
task.abort();
```

## Lifecycle integration

Start tasks in `OnModuleInit` and stop them in `OnModuleDestroy`:

```rust
use ironic::{OnModuleInit, OnModuleDestroy, LifecycleFuture, LifecycleError};
use ironic::services::scheduling::{ScheduledTask, interval};

struct BackgroundWorker {
    task: std::sync::Mutex<Option<ScheduledTask>>,
}

impl OnModuleInit for BackgroundWorker {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let task = interval(Duration::from_secs(10), || async move { /* work */ });
            *self.task.lock().unwrap() = Some(task);
            Ok(())
        })
    }
}

impl OnModuleDestroy for BackgroundWorker {
    fn on_module_destroy(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            if let Some(task) = self.task.lock().unwrap().take() {
                let _ = task.shutdown().await;
            }
            Ok(())
        })
    }
}
```

## Marker attributes

Ironic provides three marker attributes for documenting handler roles in scheduling. These
are consumed by tooling and code generation:

- `#[cron("expr")]` — declare a cron schedule
- `#[interval(ms)]` — declare a fixed-interval schedule
- `#[timeout(ms)]` — declare a timeout constraint
