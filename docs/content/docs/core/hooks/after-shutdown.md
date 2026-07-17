---
title: AfterShutdown
description: Runs after all cleanup is complete — final metrics flush, shutdown duration logging, last-chance operations.
---

# AfterShutdown

Runs after **ALL** `OnModuleDestroy` callbacks have completed. This is the **absolute last hook** before the process exits.

## When it fires

```
BeforeShutdown
    │
    ▼
OnApplicationShutdown (reverse order)
    │
    ▼
OnModuleDestroy (reverse order)
    │
    ▼
AfterShutdown  ← YOU ARE HERE
    │
    ▼
Process exits
```

At this point, every module has been destroyed. Database pools are closed. Connections are released. No providers are alive except those implementing this hook.

## The trait

```rust
pub trait AfterShutdown: Send + Sync + 'static {
    fn after_shutdown(&self) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why AfterShutdown |
|---|---|
| Flush buffered metrics to an external collector | Safe — all other code is done |
| Write a shutdown report to disk | All data is stable |
| Log total shutdown duration | Start time from BeforeShutdown, end time here |
| Send a final heartbeat/telemetry event | Last external call before exit |

## Example — metrics flush

```rust
use std::sync::Arc;

#[derive(Injectable)]
pub struct MetricsFlusher {
    registry: Arc<MetricsRegistry>,
}

impl AfterShutdown for MetricsFlusher {
    fn after_shutdown(&self) -> LifecycleFuture<'_> {
        let registry = Arc::clone(&self.registry);
        Box::pin(async move {
            let snapshot = registry.snapshot();
            // Write to file, push to collector, etc.
            std::fs::write(
                "/tmp/shutdown_metrics.json",
                &serde_json::to_string_pretty(&snapshot).unwrap_or_default(),
            ).ok();
            tracing::info!("metrics flushed — {} counters, {} histograms",
                snapshot.counters.len(), snapshot.histograms.len());
            Ok(())
        })
    }
}
```

## What you CAN'T do here

- Access the database (pools are closed by `OnModuleDestroy`)
- Make HTTP calls to your own server (it's stopped)
- Rely on any other providers (they're destroyed)

## BeforeShutdown vs AfterShutdown

| | BeforeShutdown | AfterShutdown |
|---|---|---|
| Server state | Running, accepting | Stopped |
| Providers | All alive | All destroyed |
| Can make HTTP calls? | Yes (your own endpoints) | No |
| Best for | Draining, rejection | Final cleanup, flush |
