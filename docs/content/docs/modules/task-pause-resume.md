---
title: Task Pause & Resume
description: Pause and resume scheduled tasks at runtime without killing them — deployment-safe task management.
---

# Task Pause & Resume

## What is it?

Scheduled tasks often need to pause during deployments, maintenance, or high-load periods. `ScheduledTask::pause()` stops new invocations while keeping the task alive. `resume()` starts them again.

## How to use

```rust
use ironic::services::scheduling;
use std::time::Duration;

let task = scheduling::interval(Duration::from_secs(60), || async {
    cleanup_old_data().await;
});

// During deployment
task.pause();

// After deployment
task.resume();

// Graceful shutdown
task.shutdown().await.unwrap();
```

## Task lifecycle

```
Running → pause() → Paused (skips ticks) → resume() → Running
Running → shutdown() → Stopped (cannot resume)
```

## When to pause

| Scenario | Why |
|----------|-----|
| Database migration | Don't run scheduled tasks while schema is changing |
| Deployment | Old code may not work with new data |
| High load | Reduce background work to free resources |
| Maintenance window | Planned downtime for dependencies |
