---
title: Application Lifecycle
description: How Ironic applications start, run, and shut down — with hooks for initialization and cleanup.
---

# Application Lifecycle

Every Ironic application goes through three phases: startup, runtime, and shutdown. The framework provides hooks at each phase so your code can react.

## Lifecycle phases

```
┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────┐
│  START   │───►│ OnModuleInit │───►│ OnAppBootstrap│───►│ RUNNING  │
└──────────┘    └──────────────┘    └──────────────┘    └──────────┘
                                                                  │
                                                          │
                                                          ▼
                                                   ┌──────────┐
                                                   │ SHUTDOWN │
                                                   └──────────┘
```

## Phase 1: Module initialization

When the application starts, each module's `on_module_init` hook runs in dependency order (dependencies first):

```rust
impl OnModuleInit for DatabaseModule {
    async fn on_module_init(&self) -> Result<(), LifecycleError> {
        run_migrations().await?;
        Ok(())
    }
}
```

## Phase 2: Application bootstrap

After all modules are initialized, the application bootstraps:

```rust
impl OnApplicationBootstrap for AppModule {
    async fn on_application_bootstrap(&self) -> Result<(), LifecycleError> {
        seed_default_data().await?;
        Ok(())
    }
}
```

## Phase 3: Running

The server listens for requests:

```rust
AxumAdapter::new()
    .build(app.compile())
    .unwrap()
    .listen(([0, 0, 0, 0], 3000).into(), Shutdown::new(async {
        tokio::signal::ctrl_c().await.unwrap();
        ShutdownSignal::Interrupt
    }))
    .await;
```

## Phase 4: Shutdown

When a shutdown signal is received, the framework runs cleanup in reverse order:

```rust
impl OnModuleDestroy for DatabaseModule {
    async fn on_module_destroy(&self) -> Result<(), LifecycleError> {
        close_connection_pool().await;
        Ok(())
    }
}
```

## Shutdown signals

| Signal | Source |
|--------|--------|
| `ShutdownSignal::Interrupt` | SIGINT (Ctrl+C) |
| `ShutdownSignal::Terminate` | SIGTERM |
| `ShutdownSignal::Custom(msg)` | Application-defined |

## Lifecycle hooks

| Hook | Timing | Use case |
|------|--------|----------|
| `OnModuleInit` | After module dependencies are resolved | Open connections, load data |
| `OnApplicationBootstrap` | After all modules initialized | Seed data, warm caches |
| `OnModuleDestroy` | During shutdown | Close connections, flush data |

## Error handling

If any lifecycle hook returns an error:

- **Startup**: Application startup fails with a diagnostic message
- **Shutdown**: Errors are logged but shutdown continues for remaining modules

```rust
fn definition() -> ModuleDefinition {
    ModuleDefinition::builder::<Self>()
        .on_init(|_| async {
            // This runs during module initialization
            Ok(())
        })
        .build()
}
```

## Graceful shutdown

The framework waits for in-flight requests to complete before shutting down:

```rust
AxumAdapter::new()
    .graceful_shutdown_timeout(Duration::from_secs(30))
    .build(app.compile())
```
