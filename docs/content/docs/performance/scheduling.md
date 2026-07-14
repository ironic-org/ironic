---
title: Task Scheduling
description: Run background jobs on a schedule — fixed intervals, cron expressions, with graceful shutdown.
---

# Task Scheduling

## What you'll learn

- Run tasks every N seconds with `interval()`
- Use cron expressions for complex schedules
- Start tasks at app startup, stop them cleanly on shutdown

Enable in `Cargo.toml`:

```toml
ironic = { features = ["scheduling"] }
# For cron support:
ironic = { features = ["scheduling", "cron"] }
```

---

## Interval tasks

Run a task every N seconds:

```rust
use ironic::services::scheduling::interval;
use std::time::Duration;

let task = interval(Duration::from_secs(30), || async move {
    println!("Cleaning up old data...");
    // Your background logic here
});
```

## Cron tasks

Run at specific times:

```rust
use ironic::services::scheduling::cron_schedule;

// Every day at 3 AM
let task = cron_schedule("0 3 * * *", || async move {
    generate_daily_report().await;
});

// Every Monday at 9 AM
cron_schedule("0 9 * * 1", || async move {
    send_weekly_digest().await;
});
```

### Cron syntax

```
┌── minute (0-59)
│ ┌── hour (0-23)
│ │ ┌── day of month (1-31)
│ │ │ ┌── month (1-12)
│ │ │ │ ┌── day of week (0-6, 0=Sunday)
│ │ │ │ │
* * * * *
```

| Expression | Meaning |
|------------|---------|
| `0 * * * *` | Every hour |
| `*/15 * * * *` | Every 15 minutes |
| `0 9 * * 1-5` | Weekdays at 9 AM |
| `0 0 1 * *` | Midnight on the 1st |

## Lifecycle integration

Start tasks when the app boots, stop them on shutdown:

```rust
use ironic::{OnModuleInit, OnModuleDestroy};

#[derive(Injectable)]
struct SchedulerService;

impl OnModuleInit for SchedulerService {
    async fn on_module_init(&self) {
        interval(Duration::from_secs(60), || async move {
            cleanup_expired_sessions().await;
        });
    }
}

impl OnModuleDestroy for SchedulerService {
    async fn on_module_destroy(&self) {
        // Tasks are automatically cancelled — no manual cleanup needed
    }
}
```

## Graceful shutdown

When the app shuts down:
1. The current task invocation **completes**
2. No new invocations are scheduled
3. The server stops cleanly

## Try it yourself

1. Create a task that logs "tick" every 5 seconds
2. Start the server, watch the logs
3. Press Ctrl+C — verify the task stops cleanly

## What you learned

- [x] `interval()` runs tasks on a fixed schedule
- [x] `cron_schedule()` uses cron expressions
- [x] Start tasks in `on_module_init`, they stop in `on_module_destroy`
- [x] Graceful shutdown waits for in-flight tasks
