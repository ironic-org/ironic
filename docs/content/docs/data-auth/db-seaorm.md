---
title: SeaORM
description: Use SeaORM entities, Active Record, and the CLI to generate models from an existing database.
---

# SeaORM


## Feature flag

```toml
ironic = { features = ["seaorm-postgres"] }
```

## Connection URL

```
postgres://user:password@localhost:5432/mydb
```

## Entity definition

```rust
// src/entities/user.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

## Repository

```rust
use sea_orm::*;

#[derive(Injectable)]
pub struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: i32) -> Result<user::Model, HttpError> {
        user::Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?
            .ok_or_else(|| HttpError::not_found("NOT_FOUND", format!("User {id} not found")))
    }

    pub async fn create(&self, name: &str, email: &str) -> Result<user::Model, HttpError> {
        let active = user::ActiveModel {
            name: Set(name.to_owned()),
            email: Set(email.to_owned()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        active
            .insert(&*self.db)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))
    }

    pub async fn update_email(&self, id: i32, email: &str) -> Result<(), HttpError> {
        let mut user: user::ActiveModel = self
            .find_by_id(id)
            .await?
            .into();

        user.email = Set(email.to_owned());

        user.update(&*self.db)
            .await
            .map_err(|e| HttpError::internal("DB_ERROR", e.to_string()))?;

        Ok(())
    }
}
```

## CLI

```bash
# Install sea-orm-cli
cargo install sea-orm-cli

# Generate entities from existing database
sea-orm-cli generate entity -o src/entities
```
