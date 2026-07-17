use std::sync::Arc;

use ironic::prelude::*;
use uuid::Uuid;

use crate::modules::blogs::dto::{BlogFilterDto, CreateBlogDto, UpdateBlogDto};
use crate::modules::blogs::entities::BlogPost;
use crate::modules::blogs::services::BlogService;
use crate::modules::auth::guards::JwtGuard;
use crate::modules::decorators::{Pagination, PaginationParams};
use crate::modules::exception_filters::NotFoundFilter;
use crate::modules::interceptors::TimingInterceptor;

#[controller("/api/blogs")]
#[guard(JwtGuard)]
#[exception(NotFoundFilter)]
#[middleware(RequestTracing::new())]
#[middleware(RequestLogging::new())]
#[derive(Injectable)]
pub struct BlogsController {
    service: Arc<BlogService>,
}

#[routes]
impl BlogsController {
    #[get]
    #[interceptor(TimingInterceptor)]
    #[cache(ttl_secs = 30)]
    #[api(summary = "List blog posts", tag = "Blogs")]
    async fn list(
        &self,
        #[query] filter: BlogFilterDto,
        #[decorator(Pagination)] pagination: PaginationParams,
    ) -> Result<Json<Vec<BlogPost>>, HttpError> {
        let mut posts = self.service.list(&filter)?;
        let start = ((pagination.page - 1) * pagination.size) as usize;
        let page: Vec<BlogPost> = posts.drain(start..posts.len().min(start + pagination.size as usize)).collect();
        Ok(Json(page))
    }

    #[get("/:id")]
    #[cache(ttl_secs = 60)]
    #[api(summary = "Get blog post", tag = "Blogs")]
    async fn get(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.find(id)?;
        Ok(Json(post))
    }

    #[get("/slug/:slug")]
    #[cache(ttl_secs = 60)]
    #[api(summary = "Get by slug", tag = "Blogs")]
    async fn get_by_slug(&self, #[param] slug: String) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.find_by_slug(&slug)?;
        Ok(Json(post))
    }

    #[post]
    #[interceptor(TimingInterceptor)]
    #[api(summary = "Create post", tag = "Blogs")]
    async fn create(&self, #[body] dto: CreateBlogDto) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.create(dto)?;
        Ok(Json(post))
    }

    #[put("/:id")]
    #[interceptor(TimingInterceptor)]
    #[api(summary = "Update post", tag = "Blogs")]
    async fn update(
        &self,
        #[param] id: Uuid,
        #[body] dto: UpdateBlogDto,
    ) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.update(id, dto)?;
        Ok(Json(post))
    }

    #[delete("/:id")]
    #[interceptor(TimingInterceptor)]
    #[api(summary = "Delete post", tag = "Blogs")]
    async fn delete(&self, #[param] id: Uuid) -> Result<(), HttpError> {
        self.service.delete(id)
    }

    #[post("/:id/publish")]
    #[api(summary = "Publish post", tag = "Blogs")]
    async fn publish(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.publish(id)?;
        Ok(Json(post))
    }

    #[post("/:id/unpublish")]
    #[api(summary = "Unpublish post", tag = "Blogs")]
    async fn unpublish(&self, #[param] id: Uuid) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.unpublish(id)?;
        Ok(Json(post))
    }

    #[get("/stats")]
    #[cache(ttl_secs = 120)]
    #[api(summary = "Blog statistics", tag = "Blogs")]
    async fn stats(&self) -> Result<Json<serde_json::Value>, HttpError> {
        let stats = self.service.stats()?;
        Ok(Json(serde_json::to_value(stats).unwrap()))
    }

    #[get("/:id/categories")]
    #[api(summary = "List post categories", tag = "Blogs")]
    async fn post_categories(
        &self,
        #[param] id: Uuid,
    ) -> Result<Json<Vec<crate::modules::blogs::entities::Category>>, HttpError> {
        let post = self.service.find(id)?;
        let all = self.service.categories()?;
        let cats: Vec<crate::modules::blogs::entities::Category> = all
            .into_iter()
            .filter(|c| post.category_ids.contains(&c.id))
            .collect();
        Ok(Json(cats))
    }

    #[post("/:id/categories/:category_id")]
    #[api(summary = "Add category to post", tag = "Blogs")]
    async fn add_category(
        &self,
        #[param] id: Uuid,
        #[param] category_id: Uuid,
    ) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.add_category(id, category_id)?;
        Ok(Json(post))
    }

    #[delete("/:id/categories/:category_id")]
    #[api(summary = "Remove category from post", tag = "Blogs")]
    async fn remove_category(
        &self,
        #[param] id: Uuid,
        #[param] category_id: Uuid,
    ) -> Result<Json<BlogPost>, HttpError> {
        let post = self.service.remove_category(id, category_id)?;
        Ok(Json(post))
    }
}
