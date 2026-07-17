---
title: OnGuardDenied
description: Centralized auth failure handler — brute-force detection, rate-limit counters, audit logging per guard.
---

# OnGuardDenied

Called when **any `Guard`** returns `GuardDecision::Deny`. Fires once per denial — not per guard, not per request.

## When it fires

```
Request → Middleware → Guards execute in order
                         │
                         ├─ Guard::can_activate() → Allow (continue)
                         │
                         ├─ Guard::can_activate() → Deny  ← OnGuardDenied fires HERE
                         │
                         └─ Pipeline short-circuits → 403
```

The hook receives the guard's display name. If a request goes through 3 guards and the 2nd one denies, only ONE `OnGuardDenied` fires — for the guard that denied.

## The trait

```rust
pub trait OnGuardDenied: Send + Sync + 'static {
    fn on_guard_denied(&self, guard_name: &str) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnGuardDenied |
|---|---|
| Detect brute-force login attempts | Count denials per IP per time window |
| Send alerts on unusual denial spikes | 10x normal rate → Slack/Discord alert |
| Audit log every auth failure | Compliance requirement |
| Increment per-guard denial counters | Monitor which guards reject the most |

## Example — brute-force detector

```rust
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Injectable)]
pub struct BruteForceDetector {
    counters: Mutex<HashMap<String, u64>>,
    threshold: u64,
}

impl OnGuardDenied for BruteForceDetector {
    fn on_guard_denied(&self, guard_name: &str) -> LifecycleFuture<'_> {
        let name = guard_name.to_owned();
        let threshold = self.threshold;
        Box::pin(async move {
            let mut counters = self.counters.lock().unwrap();
            let count = counters.entry(name.clone()).and_modify(|c| *c += 1).or_insert(1);
            if *count >= threshold {
                tracing::error!(
                    guard = name,
                    count = *count,
                    "brute force threshold exceeded"
                );
            }
            Ok(())
        })
    }
}
```

## Registration

```rust
LifecycleDefinition::builder::<BruteForceDetector>()
    .guard_denied()
    .build()
```

## OnGuardDenied vs OnError

| | OnGuardDenied | OnError |
|---|---|---|
| Trigger | Guard returns Deny | Any unhandled error |
| Response | 403 to client | Error status varies |
| Data | Guard name only | Error code + message |
| Best for | Auth failure monitoring | General error monitoring |
| Can prevent? | No — 403 is already sent | No — error already happened |
