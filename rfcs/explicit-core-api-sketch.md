# Explicit Core API Sketch

This is the pre-implementation contract sketch for Ironic 0.1. It intentionally uses no procedural macros. During Phase 1, this sketch becomes the compile-checked `examples/hello-world`; until then, it is reviewed for coherence against RFCs 0001–0005.

Names on builders may be refined during implementation, but changing ownership, async construction, visibility, handler erasure, pipeline order, or platform boundaries requires updating the relevant RFC.

```rust
use std::{sync::Arc, time::Duration};

use ironic::{
    app::FrameworkApplication,
    di::{DependencySet, Injectable, ProviderDefinition, ResolveError},
    http::{
        ControllerDefinition, FrameworkResponse, Json, Path, RouteDefinition,
        handler_fn,
    },
    module::{Module, ModuleDefinition},
};
use ironic_platform_axum::AxumAdapter;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct UserId(u64);

impl std::str::FromStr for UserId {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u64>()
            .map(Self)
            .map_err(|_| "user id must be an unsigned integer")
    }
}

#[derive(Clone, Debug, Serialize)]
struct UserView {
    id: u64,
    name: String,
}

trait UserRepository: Send + Sync {
    fn find(&self, id: UserId) -> Option<UserView>;
}

struct InMemoryUserRepository;

impl UserRepository for InMemoryUserRepository {
    fn find(&self, id: UserId) -> Option<UserView> {
        (id.0 == 1).then(|| UserView {
            id: 1,
            name: "Ada".to_owned(),
        })
    }
}

// RFC 0002: trait dependencies use an explicit concrete token in 0.1.
struct UserRepositoryToken(Arc<dyn UserRepository>);

struct UsersService {
    repository: Arc<UserRepositoryToken>,
}

impl UsersService {
    fn find(&self, id: UserId) -> Result<UserView, AppError> {
        self.repository.0.find(id).ok_or(AppError::UserNotFound)
    }
}

impl Injectable for UsersService {
    fn inject(dependencies: DependencySet) -> Result<Self, ResolveError> {
        Ok(Self {
            repository: dependencies.require::<UserRepositoryToken>()?,
        })
    }
}

struct UsersController {
    users: Arc<UsersService>,
}

impl UsersController {
    async fn find_one(&self, id: UserId) -> Result<Json<UserView>, AppError> {
        self.users.find(id).map(Json)
    }

    fn definition() -> ControllerDefinition {
        ControllerDefinition::new::<Self>("/users")
            .dependency::<UsersService>()
            .constructor(|dependencies| {
                Ok(Self {
                    users: dependencies.require::<UsersService>()?,
                })
            })
            .route(
                RouteDefinition::get("/:id")
                    .parameter(Path::<UserId>::new("id"))
                    .handler(handler_fn(
                        |controller: Arc<Self>, mut arguments| async move {
                            let id = arguments.take::<UserId>(0)?;
                            controller.find_one(id).await
                        },
                    )),
            )
            .build()
    }
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("user not found")]
    UserNotFound,
}

impl ironic::http::IntoFrameworkResponse for AppError {
    fn into_framework_response(self) -> FrameworkResponse {
        match self {
            Self::UserNotFound => FrameworkResponse::not_found(
                "USER_NOT_FOUND",
                "The requested user does not exist",
            ),
        }
    }
}

struct UsersModule;

impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::new::<Self>()
            .provider(ProviderDefinition::value(UserRepositoryToken(Arc::new(
                InMemoryUserRepository,
            ))))
            .provider(ProviderDefinition::injectable::<UsersService>())
            .controller(UsersController::definition())
            .build()
    }
}

struct AppModule;

impl Module for AppModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::new::<Self>()
            .import::<UsersModule>()
            .build()
    }
}

#[tokio::main]
async fn main() -> Result<(), ironic::FrameworkError> {
    let application = FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(
            AxumAdapter::new()
                .request_body_limit(1024 * 1024)
                .graceful_shutdown_timeout(Duration::from_secs(30)),
        )
        .build()
        .await?;

    application.listen("127.0.0.1:3000").await
}
```

## Contract coverage

- `AppModule` imports `UsersModule` through static type identity.
- The repository trait is injected through a concrete wrapper token.
- `UsersService` uses the synchronous constructor adapter over the async DI engine.
- The controller is DI-managed and exposes a type-erased route handler.
- The path extractor produces a concrete `UserId` before handler invocation.
- Application errors convert through the transport-neutral response contract.
- Axum is selected and configured only at the adapter boundary.
- No macro behavior is required for the application to work.

## Phase 1 compile gate

The workspace bootstrap is not complete until an executable example equivalent to this sketch passes:

```bash
cargo check -p hello-world
cargo test --workspace
```
