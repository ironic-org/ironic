use std::sync::Arc;

use ironic::prelude::*;
use uuid::Uuid;

use crate::modules::blogs::entities::Category;
use crate::modules::blogs::services::BlogService;

#[controller("/api/categories")]
#[derive(Injectable)]
pub struct CategoriesController {
    service: Arc<BlogService>,
}

#[routes]
impl CategoriesController {
    #[get]
    async fn list(&self) -> Result<Json<Vec<Category>>, HttpError> {
        let cats = self.service.categories()?;
        Ok(Json(cats))
    }

    #[post]
    async fn create(&self, #[body] dto: CreateCategoryDto) -> Result<Json<Category>, HttpError> {
        let cat = self.service.create_category(dto.name, dto.description)?;
        Ok(Json(cat))
    }

    #[delete("/:id")]
    async fn delete(&self, #[param] id: Uuid) -> Result<(), HttpError> {
        self.service.delete_category(id)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateCategoryDto {
    pub name: String,
    pub description: Option<String>,
}
