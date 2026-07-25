---
title: Module Structure
description: Internal anatomy of a feature module — controllers, services, repositories, DTOs, entities
---

# Module Structure

Every feature in Ironic is organized as a **module** — a self-contained vertical slice:

```
modules/<domain>/
├── mod.rs                 # Module definition (providers, exports)
├── controller/            # HTTP routes and request handling
│   ├── mod.rs
│   └── <domain>_controller.rs
├── services/              # Business logic
│   ├── mod.rs
│   └── <domain>_service.rs
├── repositories/          # Data access layer
│   ├── mod.rs
│   └── <domain>_repository.rs
├── dto/                   # Request/response types
│   ├── mod.rs
│   ├── create_<domain>_dto.rs
│   └── <domain>_response.rs
└── entities/              # Domain models
    ├── mod.rs
    └── <domain>.rs
```

## Module Definition (`mod.rs`)

```rust
use ironic::prelude::*;

pub struct UsersModule;

impl Module for UsersModule {
    fn definition() -> ModuleDefinition {
        ModuleDefinition::builder::<Self>()
            .provider(UsersService::provider_definition())
            .provider(UsersRepository::provider_definition())
            .controller(UsersController::definition())
            .build()
    }
}
```

## Controller (`controller/<domain>_controller.rs`)

```rust
use ironic::prelude::*;

#[controller("/users")]
pub struct UsersController {
    users_service: Arc<UsersService>,
}

#[routes]
impl UsersController {
    #[get("/")]
    async fn list(&self) -> Json<Vec<UserResponse>> {
        let users = self.users_service.list().await;
        Json(users.into_iter().map(UserResponse::from).collect())
    }

    #[post("/")]
    async fn create(&self, #[body] dto: CreateUserDto) -> Result<Json<UserResponse>, HttpError> {
        let user = self.users_service.create(dto).await?;
        Ok(Json(UserResponse::from(user)))
    }

    #[get("/{id}")]
    async fn get(&self, #[param("id")] id: u64) -> Result<Json<UserResponse>, HttpError> {
        let user = self.users_service.get(id).await?;
        Ok(Json(UserResponse::from(user)))
    }
}
```

## Service (`services/<domain>_service.rs`)

```rust
use ironic::prelude::*;

#[derive(Injectable)]
pub struct UsersService {
    repository: Arc<UsersRepository>,
}

impl UsersService {
    pub async fn list(&self) -> Vec<User> {
        self.repository.find_all().await
    }

    pub async fn create(&self, dto: CreateUserDto) -> Result<User, HttpError> {
        if self.repository.find_by_email(&dto.email).await.is_some() {
            return Err(HttpError::conflict("email already exists"));
        }
        let user = User::new(dto.name, dto.email, dto.password);
        self.repository.save(&user).await;
        Ok(user)
    }

    pub async fn get(&self, id: u64) -> Result<User, HttpError> {
        self.repository
            .find_by_id(id)
            .await
            .ok_or_else(|| HttpError::not_found("user"))
    }
}
```

## Repository (`repositories/<domain>_repository.rs`)

```rust
use ironic::prelude::*;

#[derive(Injectable)]
pub struct UsersRepository {
    pool: Arc<sqlx::PgPool>,
}

impl UsersRepository {
    pub async fn find_all(&self) -> Vec<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&*self.pool)
            .await
            .unwrap_or_default()
    }

    pub async fn find_by_id(&self, id: u64) -> Option<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id as i64)
            .fetch_optional(&*self.pool)
            .await
            .ok()?
    }

    pub async fn find_by_email(&self, email: &str) -> Option<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&*self.pool)
            .await
            .ok()?
    }

    pub async fn save(&self, user: &User) {
        sqlx::query(
            "INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3)",
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password_hash)
        .execute(&*self.pool)
        .await
        .ok();
    }
}
```

## DTOs (`dto/`)

### Create DTO

```rust
use serde::Deserialize;
use garde::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUserDto {
    #[garde(length(min = 2, max = 100))]
    pub name: String,

    #[garde(email)]
    pub email: String,

    #[garde(length(min = 8))]
    pub password: String,
}
```

### Response DTO

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            created_at: user.created_at.to_rfc3339(),
        }
    }
}
```

## Entities (`entities/<domain>.rs`)

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(name: String, email: String, password: String) -> Self {
        Self {
            id: 0,
            name,
            email,
            password_hash: bcrypt::hash(&password, 12).unwrap(),
            created_at: chrono::Utc::now(),
        }
    }
}
```

## Dependency Flow

```
Controller ──▶ Service ──▶ Repository ──▶ Database
                  │
                  ├──▶ Cache
                  │
                  └──▶ Event Bus
```

- **Controller** handles HTTP — delegates to service
- **Service** implements business logic — orchestrates repositories and external calls
- **Repository** abstracts data access — maps between entities and storage
- **DTOs** define API contracts — validated on input, serialized on output
- **Entities** are domain models — reflect database schema
