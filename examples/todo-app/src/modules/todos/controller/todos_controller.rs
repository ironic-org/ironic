use std::sync::Arc;
use uuid::Uuid;
use ironic::prelude::*;
use super::super::services::TodoService;
use crate::modules::todos::dto::{CreateTodoDto, UpdateTodoDto};
use crate::modules::todos::entities::Todo;

#[controller("/api/todos")]
#[derive(Injectable)]
pub struct TodosController {
    service: Arc<TodoService>,
}

#[routes]
impl TodosController {
    #[get]
    async fn list(
        &self,
        #[query] include_completed: Option<bool>,
    ) -> Result<Json<Vec<Todo>>, HttpError> {
        let todos = self.service.list(include_completed.unwrap_or(false)).await?;
        Ok(Json(todos))
    }

    #[get("/:id")]
    async fn get(&self, #[param] id: Uuid) -> Result<Json<Todo>, HttpError> {
        let todo = self.service.find(id).await?;
        Ok(Json(todo))
    }

    #[post]
    async fn create(&self, #[body] dto: CreateTodoDto) -> Result<Json<Todo>, HttpError> {
        let todo = self.service.create(dto).await?;
        Ok(Json(todo))
    }

    #[put("/:id")]
    async fn update(
        &self,
        #[param] id: Uuid,
        #[body] dto: UpdateTodoDto,
    ) -> Result<Json<Todo>, HttpError> {
        let todo = self.service.update(id, dto).await?;
        Ok(Json(todo))
    }

    #[delete("/:id")]
    async fn delete(&self, #[param] id: Uuid) -> Result<(), HttpError> {
        self.service.delete(id).await
    }

    #[post("/:id/toggle")]
    async fn toggle(&self, #[param] id: Uuid) -> Result<Json<Todo>, HttpError> {
        let todo = self.service.toggle(id).await?;
        Ok(Json(todo))
    }

    #[delete("/completed")]
    async fn clear_completed(&self) -> Result<Json<serde_json::Value>, HttpError> {
        let count = self.service.delete_completed().await?;
        Ok(Json(serde_json::json!({ "deleted": count })))
    }
}
