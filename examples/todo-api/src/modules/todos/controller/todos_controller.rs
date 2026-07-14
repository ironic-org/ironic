use std::sync::Arc;

use ironic::prelude::*;

use crate::modules::todos::{
    dto::{CreateTodosDto, UpdateTodosDto},
    entities::Todo,
};

use super::super::services::TodosService;

#[controller("/todos")]
#[derive(Injectable)]
pub struct TodosController {
    service: Arc<TodosService>,
}

#[routes]
impl TodosController {
    #[get]
    async fn list(&self) -> Result<Json<Vec<Todo>>, HttpError> {
        Ok(Json(self.service.list()))
    }

    #[get("/:id")]
    async fn get(&self, #[param] id: u64) -> Result<Json<Todo>, HttpError> {
        self.service.find(id).map(Json)
    }

    #[post]
    async fn create(&self, #[body] dto: CreateTodosDto) -> Result<Json<Todo>, HttpError> {
        let todo = self.service.create(dto);
        Ok(Json(todo))
    }

    #[put("/:id")]
    async fn update(
        &self,
        #[param] id: u64,
        #[body] dto: UpdateTodosDto,
    ) -> Result<Json<Todo>, HttpError> {
        self.service.update(id, dto).map(Json)
    }

    #[delete("/:id")]
    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> {
        self.service.delete(id)
    }
}
