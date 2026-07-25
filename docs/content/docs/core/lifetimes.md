---
title: Service Lifetimes
description: Control how and when services are created — Singleton, Transient, and Request-scoped providers with eager initialization.
---

# Service Lifetimes

Every provider in Ironic has a **lifetime policy** (scope) that determines
how many instances are created and when they are destroyed.

## The Three Scopes

```
       Singleton                     Transient                      Request
   ┌───────────────┐           ┌───────────────┐           ┌───────────────┐
   │   Container   │           │   Container   │           │   HTTP Request│
   │  ┌─────────┐  │           │  ┌─────────┐  │           │  ┌─────────┐  │
   │  │Service A│  │           │  │Service A│  │           │  │Service A│  │
   │  │(shared) │  │           │  │(new)    │  │           │  │(per-req)│  │
   │  └─────────┘  │           │  └─────────┘  │           │  └─────────┘  │
   │               │           │               │           │               │
   │  ┌─────────┐  │           │  ┌─────────┐  │           │  ┌─────────┐  │
   │  │Service B│──│──same────▶│  │Service B│  │           │  │Service B│  │
   │  │(shared) │  │           │  │(new)    │  │           │  │(per-req)│  │
   │  └─────────┘  │           │  └─────────┘  │           │  └─────────┘  │
   └───────────────┘           └───────────────┘           └───────────────┘
```

| Scope | Instances | Created | Destroyed | Memory | Use Case |
|-------|-----------|---------|-----------|--------|----------|
| **Singleton** | 1 per container | On first resolve (or eager) | Application shutdown | Permanent | DB pools, config, caches |
| **Transient** | 1 per `resolve()` call | Every injection | After each use (GC) | Temporary | Stateless helpers, builders |
| **Request** | 1 per HTTP request | First resolve in request | Request end | Per-request | User context, trace ID |

## Setting Scope

### Via `#[injectable]` attribute (recommended)

```rust
#[derive(Injectable)]
#[injectable(scope = "singleton")]  // default — can omit
pub struct DatabasePool;

#[derive(Injectable)]
#[injectable(scope = "transient")]
pub struct IdGenerator;

#[derive(Injectable)]
#[injectable(scope = "request")]
pub struct CurrentUser {
    pub id: u64,
    pub roles: Vec<String>,
}
```

### Via `ProviderDefinition` (for third-party types)

```rust
use ironic::{ProviderDefinition, Scope};

// Third-party type that doesn't have #[derive(Injectable)]
ProviderDefinition::constructor::<RedisClient, _, _>(
    Scope::Singleton,   // ← scope defined here
    vec![Dependency::required::<RedisConfig>()],
    |resolver| async {
        let config = resolver.resolve::<RedisConfig>().await?;
        Ok(RedisClient::new(config.url))
    },
)
```

## Singleton (Default)

**One instance per container.** All consumers share the same instance.

```rust
#[derive(Injectable)]  // scope = "singleton" is implicit
pub struct DatabasePool {
    pool: PgPool,
}

// Both services share the SAME DatabasePool instance
#[derive(Injectable)]
struct ServiceA {
    db: Arc<DatabasePool>,  // same instance
}

#[derive(Injectable)]
struct ServiceB {
    db: Arc<DatabasePool>,  // same instance
}
```

### Singleton Storage

Singletons are backed by `tokio::sync::OnceCell`:

```
First resolve:
  OnceCell is EMPTY → run factory → store result → return Arc<T>

Subsequent resolves:
  OnceCell is FULL → return clone of Arc<T>
```

### Singleton Retry

If a singleton's constructor fails or is cancelled, it can be **retried**:

```rust
#[derive(Injectable)]
#[injectable(eager)]
struct DatabasePool {
    // If startup fails, the next resolve attempt retries construction
}
```

This is safe because Rust's async model allows dropping futures
without corrupting state.

## Transient

**A new instance on every resolution.** No sharing.

```rust
#[derive(Injectable)]
#[injectable(scope = "transient")]
pub struct RequestId {
    id: String,
}

impl RequestId {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

// Each injection creates a NEW RequestId with a different UUID
fn handle(service: Arc<OrderService>) {
    let id1: Arc<RequestId> = container.resolve().await?;
    let id2: Arc<RequestId> = container.resolve().await?;
    assert_ne!(id1.id, id2.id);  // different values!
}
```

### When to Use Transient

| Situation | Transient? | Why |
|-----------|-----------|-----|
| ID generator | ✅ | Each caller needs a unique ID |
| Hashing utility | ✅ | No state to share |
| DTO mapper | ✅ | Stateless transformation |
| Database connection | ❌ | Connections are expensive — use Singleton |
| Cache service | ❌ | Cache must be shared — use Singleton |

## Request

**One instance per HTTP request.** Shared within the same request
but isolated across requests.

```rust
#[derive(Injectable)]
#[injectable(scope = "request")]
pub struct CurrentUser {
    pub id: u64,
    pub email: String,
    pub roles: Vec<String>,
}
```

### How Request Scope Works

```
Request 1                        Request 2
┌──────────────────────┐        ┌──────────────────────┐
│  RequestScope        │        │  RequestScope        │
│  ┌────────────────┐  │        │  ┌────────────────┐  │
│  │ CurrentUser    │  │        │  │ CurrentUser    │  │
│  │ id: 1          │  │        │  │ id: 2          │  │
│  │ email: "a@b"   │  │        │  │ email: "c@d"   │  │
│  └────────────────┘  │        │  └────────────────┘  │
│                      │        │                      │
│  Controller A        │        │  Controller A        │
│  └─ injects CurrentUser│       │  └─ injects CurrentUser│
│  Controller B        │        │  Controller B        │
│  └─ injects same CU  │        │  └─ injects same CU  │
└──────────────────────┘        └──────────────────────┘
     ↑                                ↑
  Different CurrentUser         Different CurrentUser
  (isolated per request)        (isolated per request)
```

### Accessing Request Scope

In HTTP handlers, the framework creates the `RequestScope` automatically.
All route handler injections go through it.

For manual resolution outside HTTP handlers:

```rust
let scope = container.request_scope();
let user = scope.resolve::<CurrentUser>().await?;
```

### Request Scope Internals

Each `RequestScope` has its own cache:

```rust
struct RequestScope {
    container: Container,
    cache: Arc<Mutex<HashMap<ProviderKey, OnceCell<ProviderValue>>>>,
}
```

- First resolve within a request → creates and caches the instance
- Subsequent resolves → returns the cached instance
- Request ends → `RequestScope` is dropped → cache is cleared

## Scope Cross-Injection Rules

```
Can inject?      Singleton    Transient    Request
──────────────────────────────────────────────────
Singleton          ✅           ✅           ❌
Transient          ✅           ✅           ✅
Request            ✅           ✅           ✅
```

### Singleton → Request (FORBIDDEN)

```rust
struct CacheWarmer {
    ctx: Arc<CurrentUser>,  // CurrentUser is request-scoped
}

// Runtime error:
// IRONIC_DI_SCOPE_VIOLATION: singleton construction cannot resolve
//   request provider `CurrentUser`
//   Chain: CacheWarmer → CurrentUser
```

**Why:** Singletons live forever. If a singleton captured request-scoped
state at startup, it would hold stale data for all subsequent requests.

**Fix:** Pass request data as method parameters instead of constructor injection.

### Singleton → Transient (SAFE)

```rust
struct MetricsService {
    timestamp: Arc<TimestampGenerator>,  // Transient is fine
}
```

Each time `MetricsService` uses `TimestampGenerator`, it gets a fresh instance
with the current time.

## Eager Initialization

By default, singletons are **lazy** — created on first use.
Use `#[injectable(eager)]` to force creation at application startup:

```rust
#[derive(Injectable)]
#[injectable(eager)]
pub struct DatabasePool {
    // Created during Application::build()
    // If the DB is unreachable, you know immediately
}
```

### Lazy vs Eager

```
Lazy Singleton:                    Eager Singleton:
                                   
Request 1 ──▶ resolve(UserService)  Startup ──▶ resolve(DatabasePool)
                │                                  │
                ├── resolve(DatabasePool)           │
                │       │                           │
                │    ┌──┴──┐                     ┌──┴──┐
                │    │ 💥? │ ← error at 3 AM    │ ✅  │ ← error at deploy
                │    └─────┘                     └─────┘
                │                                     │
                ▼                                     ▼
            500 Error                              Deploy fails
```

### Decision Guide

| Service | Eager? | Reason |
|---------|--------|--------|
| Database pool | ✅ | Fail fast on bad credentials |
| Cache connection | ✅ | Catch Redis outages at startup |
| Message queue producer | ✅ | Validate broker connectivity |
| HTTP health check | ✅ | Start responding immediately |
| Business logic service | ❌ | No startup cost — lazy is fine |
| Rarely-used service | ❌ | Save memory until needed |
| Feature flag evaluator | ❌ | Cheap to create, lazy is fine |

## Manual ProviderDefinition

When you can't use `#[derive(Injectable)]` (third-party types, complex
construction), register providers manually:

```rust
use ironic::{ProviderDefinition, Dependency, Scope};

// Constructor (sync)
ProviderDefinition::constructor::<DatabasePool, _, _>(
    Scope::Singleton,
    vec![Dependency::required::<DatabaseConfig>()],
    |resolver| {
        let config = resolver.resolve::<DatabaseConfig>()?;
        Ok(DatabasePool::new(&config.url))
    },
);

// Factory (async)
ProviderDefinition::factory::<RedisClient, _, _>(
    Scope::Singleton,
    vec![],
    |_| async {
        Ok(RedisClient::connect("redis://localhost").await?)
    },
);

// Value (pre-built)
ProviderDefinition::value(AppConfig::from_env());
```

## Performance Characteristics

| Scope | Resolution Cost | Memory | Cleanup |
|-------|----------------|--------|---------|
| Singleton | O(1) OnceCell check + Arc clone | Permanent (until shutdown) | None |
| Transient | Full construction each time | Temporary (GC after use) | Drop when Arc refs reach 0 |
| Request | O(1) cache check + Arc clone (after first) | Request lifetime | Dropped with RequestScope |

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `ScopeViolation` | Singleton → Request | Pass data as method args, not DI |
| `RequestScopeRequired` | Request provider without scope | Use `container.request_scope()` |
| `MissingProvider` | Provider not registered | Add to `module(providers = [...])` |
| `CircularDependency` | Cycle detected | Use `ForwardRef<T>` |

## What you learned

- [x] Three scopes: Singleton (default), Transient, Request
- [x] Set scope via `#[injectable(scope = "...")]` or `ProviderDefinition`
- [x] Request scope is auto-managed in HTTP handlers
- [x] Singletons cannot inject request-scoped providers
- [x] `#[injectable(eager)]` forces startup initialization
- [x] `ProviderDefinition` gives manual control for third-party types
