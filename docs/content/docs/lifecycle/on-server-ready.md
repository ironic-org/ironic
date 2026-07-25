---
title: OnServerReady
description: Server is bound and accepting connections — service discovery, health checks, metrics
---

# OnServerReady

`OnServerReady` fires after the HTTP server is **bound to its address**
and **accepting connections**. This is your cue to announce readiness.

## Position

```
OnApplicationBootstrap
  │
  ▼
OnServerReady ◀── YOU ARE HERE (server is live)
  │
  ▼
─── Request handling begins ───
```

## Trait

```rust
pub trait OnServerReady {
    async fn on_server_ready(&self) -> Result<(), LifecycleError>;
}
```

## Basic Usage

```rust
pub struct HealthAnnouncer;

impl OnServerReady for HealthAnnouncer {
    async fn on_server_ready(&self) -> Result<(), LifecycleError> {
        tracing::info!("server is ready — announcing to service discovery");

        // Register with Consul
        consul::register("api-gateway", 3000).await
            .map_err(|e| LifecycleError::new(format!("consul: {e}")))?;

        Ok(())
    }
}

// Register:
#[derive(Module)]
#[module(
    lifecycle_server_ready = [HealthAnnouncer],
)]
pub struct AppModule;
```

## Common Uses

- Register with service discovery (Consul, etcd, K8s)
- Start accepting load balancer traffic
- Initialize metrics scraping endpoints
- Warm up connection pools
- Log startup timing information
