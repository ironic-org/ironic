---
title: Modules
description: Organize your application into feature modules — the foundation of Ironic's architecture.
---

# Modules

Modules are the primary way to organize code in Ironic. Every controller, service, and configuration belongs to a module. Modules can import other modules, creating a dependency graph that the framework validates at startup.

## What is a module?

A module groups related functionality — controllers, services, providers — into a reusable unit:

```
UsersModule
├── UserController     (routes: GET/POST/PUT /users)
├── UserService        (business logic: CRUD, validation)
├── UserRepository     (data access: database queries)
└── → imports DatabaseModule
```

## Defining a module

```rust
use ironic::*;

pub struct UsersModule;

impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .import::<DatabaseModule>()
            .provider(ProviderDefinition::value(UserService::new))
            .controller(UsersController::definition())
            .export::<UserService>()
            .build()
    }
}
```

## Module structure

```
src/
├── main.rs               # Application::create().module::<AppModule>()
└── users/
    ├── mod.rs             # Module implementation
    ├── controller.rs      # Routes
    └── service.rs         # Business logic
```

### mod.rs

```rust
pub struct UsersModule;

impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .import::<DatabaseModule>()
            .provider(ProviderDefinition::value(UserService::new))
            .controller(UsersController::definition())
            .build()
    }
}
```

### controller.rs

```rust
#[controller("/users")]
pub struct UsersController {
    service: Arc<UserService>,
}

#[routes]
impl UsersController {
    #[get("/")]
    async fn list(&self) -> Json<Vec<User>> {
        Json(self.service.list().await)
    }
}
```

### service.rs

```rust
#[injectable]
pub struct UserService {
    db: Arc<dyn DatabaseProvider>,
}

impl UserService {
    pub async fn list(&self) -> Vec<User> {
        self.db.find_all().await
    }
}
```

## Module API

### `ModuleDefinition::builder()`

Creates a new module definition builder. The generic parameter is the module type:

```rust
ModuleDefinition::builder::<UsersModule>()
```

### `.import::<M>()`

Declares a dependency on another module. The imported module's exports become available:

```rust
.import::<DatabaseModule>()
```

### `.provider(definition)`

Registers a provider in the DI container:

```rust
.provider(ProviderDefinition::value(UserService::new))
```

### `.controller(definition)`

Registers a controller with its routes:

```rust
.controller(UsersController::definition())
```

### `.export::<T>()`

Makes a provider available to modules that import this one:

```rust
.export::<UserService>()
```

## Dynamic modules

Modules can be conditionally included or configured at runtime:

```rust
let mut builder = ModuleDefinition::builder::<AppModule>();

if cfg!(feature = "redis") {
    builder = builder.import::<RedisModule>();
}

builder.build()
```

## Module lifecycle hooks

Modules can implement lifecycle traits:

```rust
impl OnModuleInit for UsersModule {
    async fn on_module_init(&self) -> Result<(), LifecycleError> {
        println!("UsersModule initialized");
        Ok(())
    }
}
```

## Best practices

- **One module per feature** — Group by domain, not by layer
- **Keep modules focused** — A module should have 1-3 controllers
- **Export only what's needed** — Hide internal providers with `pub(crate)`
- **Use imports for cross-module dependencies** — Don't reference another module's internals
- **Feature-gate optional modules** — Use Cargo features for optional capabilities
