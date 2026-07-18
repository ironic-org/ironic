use std::sync::Arc;

use ironic::prelude::*;
use uuid::Uuid;

use crate::modules::auth::guards::JwtGuard;
use crate::modules::blogs::dto::CreateCategoryDto;
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
    #[api(summary = "List categories", tag = "Categories")]
    #[resp(200, "List of categories", json = Vec<Category>)]
    async fn list(&self) -> Result<Json<Vec<Category>>, HttpError> {
        let cats = self.service.categories()?;
        Ok(Json(cats))
    }

    #[post]
    #[guard(JwtGuard)]
    #[api(summary = "Create category", tag = "Categories", security = "bearerAuth")]
    #[body(json = CreateCategoryDto)]
    #[resp(201, "Category created", json = Category)]
    async fn create(&self, #[body] dto: CreateCategoryDto) -> Result<Json<Category>, HttpError> {
        let cat = self.service.create_category(dto.name, dto.description)?;
        Ok(Json(cat))
    }

    #[delete("/:id")]
    #[guard(JwtGuard)]
    #[api(summary = "Delete category", tag = "Categories", security = "bearerAuth")]
    #[resp(204, "Category deleted")]
    #[resp(404, "Category not found")]
    async fn delete(&self, #[param] id: Uuid) -> Result<(), HttpError> {
        self.service.delete_category(id)
    }
}
