---
title: Dependency Management
description: Master Ironic's dependency injection — injection scopes, optional deps, eager initialization, circular dependency detection, and provider health
---

# Dependency Management

Ironic's DI container resolves dependencies automatically. This page covers
everything from basic injection to advanced patterns.

## How Injection Works

When you annotate a struct with `#[derive(Injectable)]`, Ironic:

1. Reads each field's type (`Arc<T>`, `ForwardRef<T>`, or `Option<Arc<T>>`)
2. Generates a provider definition with the correct dependencies
3. At startup, resolves all dependencies before calling your constructor

```rust
#[derive(Injectable)]
struct OrderService {
    // Detected as: required dependency on UserRepository
    users: Arc<UserRepository>,

    // Detected as: required dependency on ForwardRef<NotificationService>
    // (breaks circular deps — see Circular Dependencies page)
    notifications: ForwardRef<NotificationService>,

    // Detected as: optional dependency on CacheService
    cache: Option<Arc<CacheService>>,
}
```

## Injection Scopes

Every provider has a **scope** that controls how many instances are created:

```
Scope         | Instances | When Created            | Best For
──────────────|───────────|────────────────────────|────────────────
Singleton     | 1         | On first use (or eager) | Services, DB pools, config
Transient     | Per-call  | Every time injected     | Stateless utilities
Request       | Per-request | On first use in request | Request-scoped state
```

```rust
// Singleton (default) — one instance shared across the app
#[derive(Injectable)]
#[injectable(scope = "singleton")]
pub struct DatabasePool;   // Created once, reused everywhere

// Transient — new instance every time
#[derive(Injectable)]
#[injectable(scope = "transient")]
pub struct RequestId;       // Fresh value for each injection

// Request — one per HTTP request
#[derive(Injectable)]
#[injectable(scope = "request")]
pub struct CurrentUser;     // Populated per-request
```

### Scope Rules

```
Singleton ──CAN inject──▶ Singleton  ✅
Singleton ──CAN inject──▶ Transient  ✅  (transient created fresh each time)
Singleton ──CAN inject──▶ Request    ❌  (not yet available)

Transient ──CAN inject──▶ Any        ✅

Request   ──CAN inject──▶ Singleton  ✅
Request   ──CAN inject──▶ Transient  ✅
Request   ──CAN inject──▶ Request    ✅ (shared within same request)
```

**Common error:** A singleton trying to inject a request-scoped provider:

```
IRONIC_DI_SCOPE_VIOLATION: singleton construction cannot resolve
request provider `CurrentUser`
```

**Fix:** Use `ModuleRef::resolve()` or pass `RequestScope` explicitly.

## Optional Dependencies

Sometimes a dependency may not be registered. Use `Option<Arc<T>>` to handle
this gracefully:

```rust
use ironic::{Injectable, ModuleRef};

#[derive(Injectable)]
struct AnalyticsService {
    // Will be None if CacheService is not registered
    cache: Option<Arc<CacheService>>,

    // ModuleRef for lazy resolution
    module_ref: Arc<ModuleRef>,
}

impl AnalyticsService {
    pub async fn track(&self, event: &str) {
        // Option 1: Direct optional access
        if let Some(cache) = &self.cache {
            cache.record_event(event).await;
        } else {
            tracing::debug!("cache not available, skipping");
        }

        // Option 2: Runtime resolution via ModuleRef
        if let Ok(cache) = self.module_ref.resolve::<CacheService>().await {
            cache.record_event(event).await;
        }
    }
}
```

### Optional vs ModuleRef

| Approach | Pros | Cons |
|----------|------|------|
| `Option<Arc<T>>` | Simple, compile-time | Must be known at startup |
| `ModuleRef::resolve()` | Runtime discovery | Slightly more code |

## Eager Initialization

By default, singletons are created **on first use** (lazy). Use `#[injectable(eager)]`
to force construction at application startup:

```rust
#[derive(Injectable)]
#[injectable(eager)]
pub struct DatabasePool {
    // Created during Application::build(), not on first query
    // Connection errors surface at startup, not at 3 AM
}

#[derive(Injectable)]
#[injectable(eager)]
pub struct MetricsCollector {
    // Starts collecting metrics immediately
    // Catches misconfigured endpoints early
}
```

**When to use eager:**

| Situation | Eager? | Reason |
|-----------|--------|--------|
| Database connection pool | ✅ | Fail fast on bad credentials |
| Cache warmup | ✅ | Pre-populate before traffic |
| Background task runner | ✅ | Start processing immediately |
| Stateless service | ❌ | No startup cost to defer |
| Rarely-used service | ❌ | Save memory until needed |

## Circular Dependency Detection

Ironic detects cycles at **resolve time** (not compile time) and reports the
full dependency chain:

```rust
struct A { b: Arc<B> }
struct B { c: Arc<C> }
struct C { a: Arc<A> }  // ← cycle back to A
```

When constructing any of these, the container returns:

```
RF_DI_CIRCULAR_DEPENDENCY: resolving `A` would create a cycle
  chain: A → B → C → A
```

### Breaking Cycles

Use `ForwardRef<T>` on one side of the cycle. See the
[Circular Dependencies](/docs/fundamentals/circular-dependencies) page
for details.

## Leaked Singleton Detection

If a singleton constructor is cancelled (e.g., the future is dropped), the
container allows a **retry**:

```rust
#[derive(Injectable)]
#[injectable(eager)]
struct ExpensiveService {
    // If construction gets cancelled by a shutdown signal,
    // the next resolution attempt will retry construction
}
```

This is different from languages with thread-local initialization —
Rust's async model allows safe retries.

## Provider Health

Monitor provider construction statistics:

```rust
use ironic::Container;

fn check_health(container: &Container) {
    let health = container.health();

    println!("Total providers: {}", health.total_providers);

    for (key, state) in &health.providers {
        println!("  {}: {} ok, {} errors",
            key.type_name(),
            state.construct_count,
            state.error_count,
        );
    }
}
```

Output:
```
Total providers: 47
  UserService: 1 ok, 0 errors
  DatabasePool: 2 ok, 1 errors  ← failed once, retried
  CacheService: 0 ok, 0 errors  ← lazy, not yet constructed
```

## Dependency Ordering

The container doesn't guarantee dependency ordering beyond "dependencies
before dependents." However, it resolves in **topological order**:

```
A ──depends on──▶ B ──depends on──▶ C
                                    ▲
D ──depends on──────────────────────┘

Resolution order: C → B → A → D
(but: A is only resolved when first requested)
```

For ordering guarantees during startup, use lifecycle hooks:

```rust
#[derive(Injectable)]
struct DatabaseMigrator {
    pool: Arc<DatabasePool>,
}

// OnModuleInit runs after ALL singletons are constructed
impl OnModuleInit for DatabaseMigrator {
    async fn on_module_init(&self) -> Result<(), LifecycleError> {
        self.pool.run_migrations().await?;
        Ok(())
    }
}
```

## Dependency Injection Patterns

### Constructor Injection (Default)

```rust
#[derive(Injectable)]
struct UserService {
    repo: Arc<UserRepository>,
    cache: Option<Arc<CacheService>>,
}
```

### Factory Injection

For complex construction logic, use a factory provider:

```rust
use ironic::{ProviderDefinition, Dependency, Scope};

pub fn database_pool_provider() -> ProviderDefinition {
    ProviderDefinition::factory::<DatabasePool, _, _>(
        Scope::Singleton,
        vec![Dependency::required::<DatabaseConfig>()],
        |resolver| async move {
            let config = resolver.resolve::<DatabaseConfig>().await?;
            let pool = DatabasePool::connect(&config.url).await
                .map_err(|e| ResolveError::factory::<DatabasePool>(e))?;
            Ok(pool)
        },
    )
}
```

### Value Injection

For pre-existing values (config, channels):

```rust
use ironic::ProviderDefinition;

let config = AppConfig::from_env();
let provider = ProviderDefinition::value(config);

Application::builder()
    .module(AppModule)
    .override_provider(provider)  // replaces existing registration
    .build()
    .await?;
```

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `RF_DI_MISSING_PROVIDER` | Provider not registered | Add to `providers = [...]` in module |
| `RF_DI_CIRCULAR_DEPENDENCY` | Cycle detected | Use `ForwardRef<T>` on one side |
| `RF_DI_SCOPE_VIOLATION` | Singleton → Request injection | Use `ModuleRef` or restructure |
| `RF_DI_DUPLICATE_PROVIDER` | Same type registered twice | Remove duplicate registration |
| `RF_DI_FACTORY_FAILED` | Constructor returned error | Check constructor logic |
| `IRONIC_DI_REQUEST_SCOPE_REQUIRED` | Request provider without scope | Call `.request_scope()` on container |

## Summary

```
                        ┌─────────────────────────┐
                        │    DI Container          │
                        │                          │
    Registration         │  ┌───────────────────┐   │    Resolution
    ────────────         │  │  Provider Registry │   │    ──────────
    #[derive(Injectable)]│  │                   │   │    container
    ProviderDefinition   │  │  Resolver ──▶ Arc │   │    .resolve()
    Module::providers[]  │  │                   │   │
                         │  │  Scopes:          │   │    request
                         │  │  S, T, R          │   │    .request_scope()
                         │  └───────────────────┘   │    .resolve()
                         └─────────────────────────┘
```

## What you learned

- [x] Injection scopes: Singleton, Transient, Request
- [x] Optional deps with `Option<Arc<T>>`
- [x] Eager initialization with `#[injectable(eager)]`
- [x] Circular dependency detection and resolution with `ForwardRef<T>`
- [x] Provider health monitoring
- [x] Factory and value provider patterns
- [x] Common error messages and fixes
