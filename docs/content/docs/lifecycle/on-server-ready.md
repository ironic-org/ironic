---
title: OnServerReady
description: Hook that fires when the HTTP server is bound and accepting connections.
---

# OnServerReady

Runs after the HTTP server binds to a port and is **ready to accept connections**.

## Use cases

- Self-health checks (is the server actually working?)
- Notifying orchestrators (Kubernetes readiness probe callback)
- Logging the bound address and port
- Triggering external service registration (service discovery)
- Sending startup notification to monitoring

## Signature

```rust
pub trait OnServerReady: Send + Sync + 'static {
    fn on_server_ready(&self) -> LifecycleFuture<'_>;
}
```

## Example

```rust
use ironic::{OnServerReady, LifecycleError};

struct HealthReporter {
    orchestrator_url: String,
}

impl OnServerReady for HealthReporter {
    async fn on_server_ready(&self) -> Result<(), LifecycleError> {
        let client = reqwest::Client::new();
        client.post(&self.orchestrator_url)
            .json(&serde_json::json!({"status": "ready"}))
            .send()
            .await
            .map_err(|e| LifecycleError::new(e.to_string()))?;
        Ok(())
    }
}
```

## When it runs

```
OnApplicationBootstrap ──► [ Server binds ] ──► OnServerReady
```

## Registration

```rust
ModuleDefinition::builder::<HealthReporter>()
    .server_ready()
    .build()
```

## Common patterns

### Logging the bound address

```rust
impl OnServerReady for ServerLogger {
    async fn on_server_ready(&self) -> Result<(), LifecycleError> {
        tracing::info!("Server ready on {}:{}", self.host, self.port);
        Ok(())
    }
}
```

### Service discovery registration

```rust
impl OnServerReady for ServiceRegistry {
    async fn on_server_ready(&self) -> Result<(), LifecycleError> {
        self.consul.register("my-service", &self.host, self.port).await
            .map_err(|e| LifecycleError::new(e.to_string()))
    }
}
```

## Error behavior

Errors are logged but do **not** stop the server — it's already running.
