---
title: Container Override
description: Hot-swap DI providers at runtime with Container::with_override() — A/B testing, feature flags, and live reconfiguration.
---

# Container Override

## What is it?

`Container::with_override()` creates a new container with one provider replaced. The original container is unchanged. Use for A/B testing, feature-flag-gated implementations, or hot-fixing a broken provider.

## How to use

```rust
let new_provider = ProviderDefinition::factory(
    ProviderKey::of::<Cache>(),
    Scope::Singleton,
    vec![],
    |_| Ok(Arc::new(RedisCache::new(redis_url))),
);

let container = container.with_override(new_provider);
```

## Use cases

| Scenario | How |
|----------|-----|
| A/B test cache backends | Override `InMemoryCache` with `RedisCache` for 10% of traffic |
| Hot-fix a broken provider | Override the failing provider with a fixed implementation |
| Feature flag gating | Override providers based on runtime feature toggles |
| Canary deployments | Gradually roll out new service implementations |

## What you learned

- [x] `with_override()` creates a new container with one replaced provider
- [x] Original container is unchanged — safe for concurrent access
- [x] Use with `Reloadable<T>` for full runtime reconfiguration
