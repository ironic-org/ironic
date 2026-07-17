---
title: OnServerReady
description: Runs after the HTTP server binds — self-health checks, orchestrator notifications, startup logging.
---

# OnServerReady

Runs after the HTTP server binds to a port and is ready to accept connections. This is the **last startup hook**.

## When it fires

```
adapter.build(arc_http)
    │
    ▼
OnServerReady  ← YOU ARE HERE
    │
    ▼
platform.listen()  ← actual TCP binding
```

At this point, every module is initialized, all providers are built, and the platform adapter is constructed. The server isn't listening yet, but it's about to.

## The trait

```rust
pub trait OnServerReady: Send + Sync + 'static {
    fn on_server_ready(&self) -> LifecycleFuture<'_>;
}
```

## When to use it

| Scenario | Why OnServerReady |
|---|---|
| Run a self-health check against your own API | Server endpoints are configured |
| Notify Kubernetes/ECS that the pod is ready | Orchestrator readiness probe |
| Log the startup time for benchmarking | Last hook before traffic arrives |
| Initialize WebSocket connection pools | Platform is built, external connections OK |

## Example — readiness probe

```rust
#[derive(Injectable)]
pub struct ReadinessReporter {
    http: reqwest::Client,
}

impl OnServerReady for ReadinessReporter {
    fn on_server_ready(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let resp = self.http
                .get("http://localhost:3000/health")
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    tracing::info!("health check passed: {}", r.status());
                }
                other => {
                    tracing::warn!("health check result: {:?}", other.map(|r| r.status()));
                }
            }
            Ok(())
        })
    }
}
```

## OnServerReady vs OnApplicationBootstrap

| | OnApplicationBootstrap | OnServerReady |
|---|---|---|
| Server state | Not built yet | Built, not listening |
| HTTP calls | Can't call own endpoints | Can call health checks |
| Timing | After all init | Right before listen() |
| Best for | Background tasks, cron | Readiness checks, notifications |
