---
title: Container Override
description: Hot-swap DI providers at runtime with Container::with_override() — A/B testing, feature flags, and live reconfiguration.
---

# Container Override

`Container::with_override()` creates a **new container** with one provider
replaced. The original container is **unchanged**, so you can maintain
multiple container versions concurrently.

## The Problem

In production, you may need to swap implementations without restarting:

```
InMemoryCache  ──▶  RedisCache    (migrate to distributed caching)
V1 Service     ──▶  V2 Service    (gradual rollout)
MockProvider   ──▶  RealProvider  (A/B test)
```

Without override, you'd need to restart the entire application.

## How `with_override` Works

```
Original Container                    Override Container
┌─────────────────────┐              ┌─────────────────────┐
│ CacheService (mem)  │              │ CacheService (mem)  │
│ UserService         │  with_override│ UserService         │
│ DatabasePool        │ ────────────▶│ DatabasePool        │
│ Config              │              │ Config              │
│                     │              │                     │
│ Registrations are   │              │ ONE registration    │
│ COPIED from orig    │              │ replaced            │
└─────────────────────┘              └─────────────────────┘
```

Key properties:
- **Immutable original** — Other references to the original container still work
- **Copy-on-write** — Only changed providers allocate new memory
- **Thread-safe** — Both containers can be used concurrently

## Basic Usage

```rust
use ironic::{Container, ProviderDefinition, ProviderKey, Scope};

// Original container has InMemoryCache
let original: Container = app.container().clone();

// Create overridden container with RedisCache
let overridden = original.with_override(
    ProviderDefinition::factory::<CacheService, _, _>(
        Scope::Singleton,
        vec![],
        |_| async { Ok(RedisCache::new("redis://localhost")) },
    ),
);

// Original unaffected
let mem_cache = original.resolve::<CacheService>().await?;   // InMemoryCache
let redis_cache = overridden.resolve::<CacheService>().await?; // RedisCache
```

## A/B Testing Pattern

Route a percentage of traffic to an overridden container:

```rust
use std::sync::Arc;
use ironic::{Container, ProviderDefinition};

async fn handle_request(
    orig_container: &Container,
    user_id: u64,
) -> Result<(), Error> {
    // 10% of users get the new implementation
    let container = if user_id % 10 == 0 {
        orig_container.with_override(
            ProviderDefinition::value(NewRecommendationEngine)
        )
    } else {
        orig_container.clone()  // no allocation — just Arc bump
    };

    let engine = container.resolve::<RecommendationEngine>().await?;
    engine.recommend(user_id).await
}
```

## Feature Flag Gating

Override providers based on runtime configuration:

```rust
use ironic::{Container, ProviderDefinition, Scope};

fn apply_feature_flags(
    container: &Container,
    flags: &FeatureFlags,
) -> Container {
    let mut result = container.clone();

    if flags.enable_new_payment_gateway {
        result = result.with_override(
            ProviderDefinition::value(NewPaymentGateway)
        );
    }

    if flags.use_redis_cache {
        result = result.with_override(
            ProviderDefinition::factory::<CacheService, _, _>(
                Scope::Singleton,
                vec![],
                |_| async { Ok(RedisCache::new("redis://localhost")) },
            ),
        );
    }

    result
}
```

## Hot-Fix Pattern

Quickly swap out a failing provider without restart:

```rust
use ironic::{Container, ProviderDefinition};

async fn hotfix_provider(
    container: &Container,
) -> Container {
    match container.resolve::<CriticalService>().await {
        Ok(_) => container.clone(),                    // healthy
        Err(_) => container.with_override(              // broken → hot-fix
            ProviderDefinition::value(FallbackService)
        ),
    }
}
```

## With `Reloadable<T>`

Combine with `Reloadable<T>` for full runtime reconfiguration:

```rust
use ironic::{Container, ProviderDefinition, services::config::Reloadable};

async fn watch_and_reload(
    container: Container,
    reloadable: Reloadable<AppConfig>,
) {
    while let Some(new_config) = reloadable.changed().await {
        let overridden = container.with_override(
            ProviderDefinition::value(new_config.clone())
        );
        // Store overridden container for new requests
        CURRENT_CONTAINER.store(Arc::new(overridden));
    }
}
```

## Performance

| Aspect | Cost |
|--------|------|
| `with_override()` call | O(n) where n = registered providers (clone `HashMap`) |
| Memory | One extra `Arc<Registration>` per override |
| Resolution on overridden | Same as original + 1 HashMap lookup |
| Original container | Zero impact — unchanged |

For hot-path overrides (every request), pre-create containers and cache them:

```rust
// Pre-create A/B containers at startup
let container_a = base.with_override(ProviderDefinition::value(ImplA));
let container_b = base.with_override(ProviderDefinition::value(ImplB));

// Route requests — no per-request allocation
let container = if use_a { &container_a } else { &container_b };
```

## Limitations

| Limitation | Workaround |
|-----------|------------|
| Can only replace existing providers | Use `Container::extend()` for adding new ones |
| No rollback after override | Keep original container reference for fallback |
| Overridden container is independent | Changes don't propagate to other overrides |
| Singleton state is reset | Override creates new singleton instance |

## Common Patterns

```
Startup
  │
  ├── Create base container
  │
  ├── Apply feature flags ──▶ overridden container
  │
  ├── Create A/B test variants
  │   ├── variant_a = base.with_override(ImplA)
  │   └── variant_b = base.with_override(ImplB)
  │
  └── Serve requests using appropriate variant

Runtime
  │
  ├── Feature flag changes → new override
  ├── Hot-fix triggered   → new override
  └── Config reloaded     → new override via Reloadable
```

## What you learned

- [x] `with_override()` creates a new container with one provider replaced
- [x] Original container is unchanged — safe for concurrent access
- [x] Use for A/B testing, feature flags, canary deployments, hot-fixes
- [x] Combine with `Reloadable<T>` for full runtime reconfiguration
- [x] Pre-create containers for hot-path performance
