use super::super::services::ExampleService;
use crate::modules::example::dto::{CreateExampleDto, UpdateExampleDto};
use crate::modules::example::entities::Example;
use ironic::prelude::*;
use std::sync::Arc;

#[controller("/example")]
#[derive(Injectable)]
pub struct ExampleController {
    service: Arc<ExampleService>,
}

#[routes]
impl ExampleController {
    #[get]
    async fn list(&self) -> Result<Json<Vec<Example>>, HttpError> {
        Ok(Json(self.service.list()))
    }

    #[get("/:id")]
    async fn get(&self, #[param] id: u64) -> Result<Json<Example>, HttpError> {
        self.service.find(id).map(Json)
    }

    #[post]
    async fn create(&self, #[body] dto: CreateExampleDto) -> Result<Json<Example>, HttpError> {
        Ok(Json(self.service.create(dto)))
    }

    #[put("/:id")]
    async fn update(
        &self,
        #[param] id: u64,
        #[body] dto: UpdateExampleDto,
    ) -> Result<Json<Example>, HttpError> {
        self.service.update(id, dto).map(Json)
    }

    #[delete("/:id")]
    async fn delete(&self, #[param] id: u64) -> Result<(), HttpError> {
        self.service.delete(id)
    }
}
