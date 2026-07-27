---
title: Events (Transport Provider)
description: Send and receive events across services using Kafka, Redis, or in-memory transport — no boilerplate.
---

# Events — Cross-Service Communication

Ironic's transport provider lets you send and receive events between microservices without writing boilerplate. It wraps Kafka, Redis pub/sub, or an in-memory channel behind a single `EventClient` / `EventServer` interface.

## When to use this

| Situation | What happens |
|---|---|
| Your app needs to **emit** events (e.g. `user.created`) | `EventClient` — call `.emit()` anywhere |
| Your app needs to **handle** events from other services | `#[event_handler(transport = "...")]` — one attribute |
| You want to switch from Kafka to Redis later | Change `TransportKind` in config, nothing else |

## Quick start (2 minutes)

### 1. Enable features

```toml
[dependencies]
ironic = { version = "1.0", features = [
    "microservices",   # transport provider
    "events",          # event handler macro
    "transport-kafka", # or "transport-redis"
] }
```

### 2. Create a module with transport providers

```rust
use ironic::*;

#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer],
    lifecycle_bootstrap = [EventClient, EventServer],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct EventsModule;
```

That's it. `TransportConfig` reads env vars automatically (`KAFKA_BROKERS`, `KAFKA_TOPIC`, `KAFKA_GROUP_ID`). The lifecycle hooks connect the client / start the server on boot, and disconnect on shutdown.

### 3. Handle incoming events

```rust
use ironic::event_handler;

#[event_handler(transport = "user.created", auto_register)]
async fn on_user_created(event: UserCreated) {
    tracing::info!("new user: {:?}", event);
}
```

The `transport = "user.created"` tells the macro to register this handler for the pattern `"user.created"`. The `auto_register` flag tells the framework to automatically register the handler with the `EventServer` during startup — no manual wiring needed.

The handler function receives the deserialized event. The macro handles the JSON deserialization and error handling automatically.

### 4. Emit events from anywhere

```rust
#[derive(Injectable)]
pub struct UserService {
    events: Arc<EventClient>,
}

impl UserService {
    pub async fn create_user(&self, name: String) -> Result<(), TransportError> {
        let event = UserCreated { name };
        self.events.emit("user.created", &event).await
    }
}
```

Just inject `Arc<EventClient>` into any service. The underlying transport (Kafka, Redis, in-memory) is invisible.

## Configuration

### Environment variables

| Variable | Default | What it sets |
|---|---|---|
| `KAFKA_BROKERS` | `127.0.0.1:9092` | Broker address(es) |
| `KAFKA_TOPIC` | `ironic-events` | Topic / channel name |
| `KAFKA_GROUP_ID` | `default` | Consumer group ID |

### Override config programmatically

```rust
let config = TransportConfig {
    kind: TransportKind::Kafka,
    brokers: "broker-1:9092,broker-2:9092".into(),
    topic: "my-app-events".into(),
    group_id: "my-service".into(),
};

// Pass to the application builder:
ApplicationBuilder::default()
    .module(/* your module */)
    .override_provider(ProviderDefinition::value(config))
    .build()
    .await
```

### Pick the transport backend

Use `TransportKind` to choose the backend:

```rust
TransportKind::Kafka     // requires "transport-kafka" feature
TransportKind::Redis     // requires "transport-redis" feature
TransportKind::InMemory  // always available, for testing & single-process
```

## Architecture

### How it works step-by-step

```
                  ┌──────────────────────┐
                  │    DI Container      │
                  │                      │
                  │  TransportConfig     │ ← reads env vars
                  │  EventClient         │ ← wraps MicroserviceClient
                  │  EventServer         │ ← wraps MicroserviceServer
                  └──────┬───────────────┘
                         │
           ┌─────────────┴─────────────┐
           │                           │
           ▼                           ▼
   ┌───────────────┐         ┌─────────────────┐
   │  EventClient  │         │   EventServer   │
   │  .emit(...)   │         │  .on_event(...) │
   │               │         │  .listen()      │
   └───────┬───────┘         └────────┬────────┘
           │                          │
           ▼                          ▼
   ┌───────────────┐         ┌─────────────────┐
   │ Kafka / Redis │◄───────►│   Kafka / Redis │
   │   Producer    │         │    Consumer     │
   └───────────────┘         └─────────────────┘
```

**Startup order** (automatic):

1. `Container` creates `TransportConfig` (reads env vars)
2. `Container` creates `EventClient` + `EventServer` from config
3. `AsyncModuleInit` runs — `#[event_handler(auto_register)]` registers handlers on `EventServer`
4. `OnApplicationBootstrap` — `EventClient` connects, `EventServer` starts listening
5. Events flow
6. `OnApplicationShutdown` — both disconnect gracefully

### The `auto_register` flag

Without `auto_register`, you manually call the registration function:

```rust
#[event_handler(transport = "user.created")]
async fn handle_user(event: UserCreated) { /* ... */ }

// Manual registration in some init function:
let server: Arc<EventServer> = container.resolve().await.unwrap();
__event_handler_reg_handle_user(&server);
```

With `auto_register`, the framework does the above automatically — the generated `__EventHandlerAuto_handle_user` struct implements `AsyncModuleInit`, and the framework calls it during startup.

## Full example

A complete microservice that emits events on user creation and handles them in another module:

```rust
use ironic::*;

// ── Events ──
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct UserCreated {
    name: String,
}

// ── Consumer module ──
mod consumer {
    use super::*;

    #[event_handler(transport = "user.created", auto_register)]
    async fn on_user_created(event: UserCreated) {
        tracing::info!("received: {:?}", event);
    }
}

// ── Producer service ──
#[derive(Injectable)]
pub struct UserService {
    events: Arc<EventClient>,
}

impl UserService {
    pub async fn signup(&self, name: &str) -> Result<(), TransportError> {
        self.events
            .emit("user.created", &UserCreated { name: name.into() })
            .await
    }
}

// ── Module ──
#[derive(Module)]
#[module(
    providers = [TransportConfig, EventClient, EventServer, UserService],
    lifecycle_bootstrap = [EventClient, EventServer],
    lifecycle_shutdown = [EventClient, EventServer],
)]
pub struct AppModule;
```

When `UserService::signup()` is called:
1. It emits `UserCreated` on Kafka (or Redis/InMemory)
2. The `EventServer` receives it
3. `on_user_created()` is called with the deserialized event

## Testing without infrastructure

No Kafka or Redis needed for tests:

```rust
#[tokio::test]
async fn test_event_flow() {
    let (client, server) = EventServer::paired(16);

    let received = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let events = received.clone();
    server.on_event("test", Arc::new(move |payload, _ctx| {
        let events = events.clone();
        Box::pin(async move {
            let msg: String = serde_json::from_slice(&payload).unwrap();
            events.lock().await.push(msg);
            Ok(())
        })
    }));
    server.listen().await.unwrap();

    client.emit("test", &"hello".to_string()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(received.lock().await.len(), 1);
}
```

`EventServer::paired(capacity)` creates a connected client+server pair that talks over in-memory channels. No network needed.

## Full configuration reference

### `TransportConfig`

| Field | Type | Description |
|---|---|---|
| `kind` | `TransportKind` | Backend: `Kafka`, `Redis`, or `InMemory` |
| `brokers` | `String` | Connection URL / broker list |
| `topic` | `String` | Topic or channel name |
| `group_id` | `String` | Consumer group (ignored by Redis/InMemory) |

### `EventClient`

| Method | Description |
|---|---|
| `emit(pattern, event)` | Fire-and-forget event |
| *(also implements send/close via `MicroserviceClient`)* | |

### `EventServer`

| Method | Description |
|---|---|
| `on_event(pattern, handler)` | Register handler (called by `#[event_handler]`) |
| `on_message(pattern, handler)` | Register request-response handler |
| `listen()` | Start consuming (called by lifecycle) |
| `close()` | Stop consuming (called by lifecycle shutdown) |

## Switching transports

Change one line:

```rust
// Before: Kafka
let config = TransportConfig {
    kind: TransportKind::Kafka,
    // ...
};

// After: Redis (add "transport-redis" feature)
let config = TransportConfig {
    kind: TransportKind::Redis,
    // Note: `brokers` becomes Redis URL like "redis://127.0.0.1:6379"
    // ...
};
```

No code changes to your event handlers or emitters.

## Feature flags

| Feature | What it enables |
|---|---|
| `microservices` | Transport provider (`EventClient`, `EventServer`, `TransportConfig`) |
| `events` | `#[event_handler]` proc macro |
| `transport-kafka` | Kafka backend |
| `transport-redis` | Redis pub/sub backend |

You need `microservices` + `events` + at least one transport backend for the full flow.
