---
title: Task Scheduling
description: Run background tasks at fixed intervals or on cron schedules.
---

# Task Scheduling

## What you'll learn

- Run recurring tasks with `interval()`
- Schedule tasks with cron expressions using `cron()` and `cron_schedule()`
- Gracefully shut down scheduled tasks

---

## Enabling

```toml
ironic = { features = ["scheduling"] }
```

For cron support, add `"cron"`:

```toml
ironic = { features = ["scheduling", "cron"] }
```

---

## Fixed-interval tasks

Use `interval()` to run a task at a fixed period. The first run follows one full period. If a run takes longer than the period, subsequent runs are skipped (`MissedTickBehavior::Skip`):

```rust
use ironic::services::scheduling;
use std::time::Duration;

let task = scheduling::interval(Duration::from_secs(60), || async move {
    cleanup_old_data().await;
});
```

## Cron tasks

Use `cron()` with a 6-field cron expression (second minute hour day-of-month month day-of-week):

```rust
use ironic::services::scheduling;

scheduling::cron("0 3 * * * *", || async move {
    generate_daily_report().await;
});
```

For expressions that might be invalid at runtime, use the fallible `cron_schedule()`:

```rust
use ironic::services::scheduling;

let task = scheduling::cron_schedule("0 0 * * * *", || async move {
    archive_logs().await;
}).map_err(|e| format!("invalid cron expression: {e}"))?;
```

## Graceful shutdown

Each scheduling function returns a `ScheduledTask` that can be stopped cooperatively:

```rust
use ironic::services::scheduling::ScheduledTask;
use std::time::Duration;

let task = scheduling::interval(Duration::from_secs(30), || async move {
    poll_queue().await;
});

// Later — in a shutdown hook:
task.shutdown().await.unwrap();
```

`shutdown()` requests cancellation, waits for the current invocation to finish, and returns a join error if the task panicked. For immediate termination, use `abort()`:

```rust
task.abort();
```

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Task never runs | Verify `scheduling` feature is enabled in `Cargo.toml` |
| Cron expression ignored | Enable the `cron` feature — `cron()` and `cron_schedule()` require it |
| Panic in a task kills the process | The `JoinHandle` captures panics — check the result of `shutdown().await` |
| Forgetting to hold the `ScheduledTask` | The task is aborted when `ScheduledTask` is dropped |

## What you learned

- [x] `interval(period, task)` runs at a fixed period with skip-on-backlog
- [x] `cron(expression, task)` runs on a cron schedule (requires `cron` feature)
- [x] `cron_schedule(expression, task)` is the fallible variant
- [x] `ScheduledTask::shutdown()` waits for completion; `abort()` stops immediately
- [x] Task is cancelled on drop if not explicitly shut down
