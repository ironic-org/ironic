use std::sync::Arc;
use uuid::Uuid;
use ironic::prelude::*;
use tracing::instrument;
use crate::modules::todos::dto::{CreateTodoDto, UpdateTodoDto};
use crate::modules::todos::entities::Todo;
use crate::modules::todos::repositories::TodoRepository;

#[derive(Injectable)]
pub struct TodoService {
    repository: Arc<TodoRepository>,
}

impl TodoService {
    #[instrument(skip(self))]
    pub async fn list(&self, include_completed: bool) -> Result<Vec<Todo>, HttpError> {
        self.repository.list(include_completed).await
    }

    #[instrument(skip(self))]
    pub async fn find(&self, id: Uuid) -> Result<Todo, HttpError> {
        self.repository
            .find(id)
            .await?
            .ok_or_else(|| HttpError::not_found("TODO_NOT_FOUND", format!("Todo {id} not found")))
    }

    #[instrument(skip(self))]
    pub async fn create(&self, dto: CreateTodoDto) -> Result<Todo, HttpError> {
        self.repository.create(&dto.title, &dto.description).await
    }

    #[instrument(skip(self))]
    pub async fn update(&self, id: Uuid, dto: UpdateTodoDto) -> Result<Todo, HttpError> {
        let current = self.find(id).await?;
        let title = dto.title.unwrap_or(current.title);
        let description = dto.description.or(current.description);
        let completed = dto.completed.unwrap_or(current.completed);
        self.repository.update(id, &title, &description, completed).await
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), HttpError> {
        let deleted = self.repository.delete(id).await?;
        if deleted {
            tracing::info!(todo_id = %id, "todo deleted");
            Ok(())
        } else {
            Err(HttpError::not_found("TODO_NOT_FOUND", format!("Todo {id} not found")))
        }
    }

    #[instrument(skip(self))]
    pub async fn toggle(&self, id: Uuid) -> Result<Todo, HttpError> {
        self.find(id).await?;
        self.repository.toggle(id).await
    }

    #[instrument(skip(self))]
    pub async fn delete_completed(&self) -> Result<u64, HttpError> {
        let count = self.repository.delete_completed().await?;
        tracing::info!(count, "cleared completed todos");
        Ok(count)
    }
}
