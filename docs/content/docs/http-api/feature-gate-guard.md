---
title: Feature Gate Guard
description: Gate routes behind runtime feature toggles with FeatureGateGuard — enable/disable endpoints without restarting.
---

# Feature Gate Guard

## What is it?

`FeatureGateGuard` is a built-in `Guard` that checks a named feature flag on every request. If the flag is disabled, the route returns 403. Toggle the flag at runtime via `FeatureToggle` — no restart needed.

## How to use

```rust
#[controller("/beta")]
#[guard(FeatureGateGuard::new("dark-mode"))]
#[derive(Injectable)]
struct BetaController;
```

When `dark-mode` is `false` in the feature toggle config, requests to `/beta/*` return 403. When enabled, requests flow through normally.

## Runtime toggle

```rust
let mut toggles = FeatureToggle::default();
toggles.set("dark-mode", true);
```

For hot-reload, register the watcher:

```rust
let watcher = ConfigWatcher::new(config);
let toggle = FeatureToggle::from_root_config(&config).unwrap()
    .with_watcher(watcher);
```

## Use for

| Scenario | Example |
|----------|---------|
| Gradual rollout | `#[guard(FeatureGateGuard::new("new-checkout"))]` |
| Beta features | `#[guard(FeatureGateGuard::new("beta-dashboard"))]` |
| Maintenance mode | `#[guard(FeatureGateGuard::new("api-v2"))]` |
| Kill switch | `#[guard(FeatureGateGuard::new("payment-processing"))]` |

## What you learned

- [x] `FeatureGateGuard` gates routes behind runtime toggles
- [x] Use `FeatureToggle::set()` or `ConfigWatcher` for hot-reload
- [x] No restart needed to enable/disable endpoints
