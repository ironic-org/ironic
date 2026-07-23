---
title: Providers
description: Register and resolve dependencies with Ironic's type-safe DI container.
---

# Providers

Providers are the building blocks of Ironic's dependency injection system. A provider knows how to create a value of a specific type and manages its lifecycle.

## Provider scopes

| Scope | Behavior | Use case |
|-------|----------|----------|
| `Singleton` | One instance shared across the entire application | Database pools, config, caches |
| `Transient` | New instance created every time it's resolved | Stateless services, DTOs |
| `Request` | One instance per HTTP request (scoped) | Request context, DB transactions |

## Defining providers

### From a value

```rust
ProviderDefinition::value(AppConfig::default())
```

Always creates a `Singleton`.

### From a factory (async)

```rust
ProviderDefinition::factory(
    Scope::Singleton,
    vec![Dependency::required::<DbPool>()],
    |resolver| async move {
        let pool = resolver.resolve::<DbPool>().await?;
        Ok(UserRepository::new(pool))
    },
)
```

### From a constructor (sync)

```rust
ProviderDefinition::constructor(
    Scope::Transient,
    Vec::new(),
    |_resolver| Ok(EmailService::new()),
)
```

## Registration

Providers are registered in modules:

```rust
impl Module for AppModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .provider(ProviderDefinition::value(
                CacheService::new(60),
            ))
            .provider(ProviderDefinition::constructor(
                Scope::Singleton,
                vec![Dependency::required::<CacheService>()],
                |resolver| async {
                    let cache = resolver.resolve::<CacheService>().await?;
                    Ok(UserService::new(cache))
                },
            ))
            .build()
    }
}
```

## Resolution

### In controllers

```rust
#[controller("/users")]
struct UsersController {
    service: Arc<UserService>,  // resolved by DI
}
```

### Manual resolution

```rust
let container = app.container();
let service = container.resolve::<UserService>().await?;
```

## Dependencies

Providers declare their dependencies explicitly:

```rust
Dependency::required::<DatabasePool>()
Dependency::optional::<CacheService>()
```

The resolver uses these declarations to:
1. Build the dependency graph
2. Detect cycles at resolution time
3. Report missing providers with a diagnostic path

## Provider health

Each provider tracks construction statistics:

```rust
let health = container.health();
for (key, stats) in health.providers {
    println!("{}: {} OK, {} errors", key, stats.construct_count, stats.error_count);
}
```

## Overriding in tests

```rust
let mut builder = ContainerBuilder::new();
builder
    .register(ProviderDefinition::value(RealService::new()))
    .unwrap()
    .override_with(ProviderDefinition::value(MockService::new()))
    .unwrap();
```

## Provider resolution order

1. Check if already resolved (singleton/request cache)
2. Check registration exists
3. Check for circular dependencies
4. Resolve dependencies recursively
5. Construct the provider
6. Cache if singleton or request-scoped
7. Return the value

## Error scenarios

| Error | Cause |
|-------|-------|
| `MissingProvider` | Provider not registered |
| `CircularDependency` | A → B → A cycle detected |
| `FactoryFailed` | Factory returned an error |
| `TypeMismatch` | Resolved type doesn't match requested type |
| `RequestScopeRequired` | Request-scoped provider resolved outside a request |
| `ScopeViolation` | Singleton tries to depend on request-scoped provider |
