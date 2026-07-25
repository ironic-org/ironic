---
title: OnApplicationBootstrap
description: Cross-module coordination after all modules have initialized
---

# OnApplicationBootstrap

`OnApplicationBootstrap` fires after **ALL modules** have completed their
`OnModuleInit` hooks. This is the last startup hook before the server
starts listening.

## Position

```
OnModuleInit (all modules)
  │
  ▼
OnApplicationBootstrap ◀── YOU ARE HERE
  │
  ▼
OnServerReady
```

## Trait

```rust
pub trait OnApplicationBootstrap {
    async fn on_application_bootstrap(&self) -> Result<(), LifecycleError>;
}
```

## Basic Usage

```rust
pub struct ServiceRegistry;

impl OnApplicationBootstrap for ServiceRegistry {
    async fn on_application_bootstrap(&self) -> Result<(), LifecycleError> {
        // All modules are initialized — safe to coordinate across them
        tracing::info!("all modules initialized");
        Ok(())
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_bootstrap = [ServiceRegistry],
)]
pub struct AppModule;
```

## When to Use

| Scenario | Why Here |
|----------|----------|
| Register with Consul/etcd | All services are ready to serve |
| Cross-module validation | Every module has initialized its state |
| Emit "startup complete" event | Guaranteed last startup hook |
| Start accepting traffic signals | Server will start accepting immediately after |
