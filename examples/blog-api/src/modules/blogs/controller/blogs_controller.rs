use std::sync::Arc;

use ironic::prelude::*;
use uuid::Uuid;

use crate::modules::blogs::dto::{BlogFilterDto, CreateBlogDto, UpdateBlogDto};
use crate::modules::blogs::entities::BlogPost;
use crate::modules::blogs::services::BlogService;

#[controller("/api/blogs")]
#[derive(Injectable)]
pub struct BlogsController {
    service: Arc<BlogService>,
}

#[routes]
impl BlogsController {
    #[get]
    async fn list(
        &self,
        #[query] filter: BlogFilterDto,
    ) -> Result<Json<Vec<BlogPost>>, HttpError> {
        let posts = self.service.list(&filter)?;
        Ok(Json(posts))
    }

    #[get("/:id")]
    async fn get(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.find(id)?;
        Ok(Json(post))
    }

    #[get("/slug/:slug")]
    async fn get_by_slug(&self, #[param] slug: String) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.find_by_slug(&slug)?;
        Ok(Json(post))
    }

    #[post]
    async fn create(&self, #[body] dto: CreateBlogDto) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.create(dto)?;
        Ok(Json(post))
    }

    #[put("/:id")]
    async fn update(
        &self,
        #[param] id: Uuid,
        #[body] dto: UpdateBlogDto,
    ) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.update(id, dto)?;
        Ok(Json(post))
    }

    #[delete("/:id")]
    async fn delete(&self, #[param] id: Uuid) -> Result<(), HttpError> {
        self.service.delete(id)
    }

    #[post("/:id/publish")]
    async fn publish(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.publish(id)?;
        Ok(Json(post))
    }

    #[post("/:id/unpublish")]
    async fn unpublish(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.unpublish(id)?;
        Ok(Json(post))
    }

    #[get("/stats")]
    async fn stats(&self) -> Result<Json<serde_json::Value>, HttpError> {
        let stats = self.service.stats()?;
        Ok(Json(serde_json::to_value(stats).unwrap()))
    }

    // --- Category sub-resource ---

    #[get("/:id/categories")]
    async fn post_categories(&self, #[param] id: Uuid) -> Result<Json<Vec<crate::modules::blogs::entities::Category>>, HttpError> {
        let post = self.service.find(id)?;
        let all = self.service.categories()?;
        let cats: Vec<crate::modules::blogs::entities::Category> = all
            .into_iter()
            .filter(|c| post.category_ids.contains(&c.id))
            .collect();
        Ok(Json(cats))
    }

    #[post("/:id/categories/:category_id")]
    async fn add_category(
        &self,
        #[param] id: Uuid,
        #[param] category_id: Uuid,
    ) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.add_category(id, category_id)?;
        Ok(Json(post))
    }

    #[delete("/:id/categories/:category_id")]
    async fn remove_category(
        &self,
        #[param] id: Uuid,
        #[param] category_id: Uuid,
    ) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.remove_category(id, category_id)?;
        Ok(Json(post))
    }
}
