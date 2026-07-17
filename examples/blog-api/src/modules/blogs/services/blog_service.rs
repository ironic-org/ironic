use std::sync::Arc;

use chrono::Utc;
use ironic::{LifecycleFuture, OnModuleInit};
use ironic::prelude::*;
use uuid::Uuid;

use crate::modules::blogs::dto::{BlogFilterDto, CreateBlogDto, UpdateBlogDto};
use crate::modules::blogs::entities::{BlogPost, Category};
use crate::modules::blogs::repositories::{BlogRepository, BlogStats, CategoryRepository};

#[derive(Injectable)]
pub struct BlogService {
    pub(crate) blog_repo: Arc<BlogRepository>,
    pub(crate) category_repo: Arc<CategoryRepository>,
}

impl OnModuleInit for BlogService {
    fn on_module_init(&self) -> LifecycleFuture<'_> {
        Box::pin(async move {
            let existing = self.blog_repo.list(&BlogFilterDto::default()).unwrap_or_default();
            if !existing.is_empty() {
                tracing::info!("{} existing blog posts found — skipping seed", existing.len());
                return Ok(());
            }

            let rust = self.create_category("Rust".into(), None);
            let framework = self.create_category("Framework".into(), None);
            let tutorial = self.create_category("Tutorial".into(), None);

            if let (Ok(rust_cat), Ok(fw_cat), Ok(tut_cat)) = (rust, framework, tutorial) {
                let _ = self.create(CreateBlogDto {
                    title: "Getting Started with Ironic".into(),
                    content: "Ironic is a type-safe Rust framework for building APIs with compile-time guarantees.".into(),
                    excerpt: Some("Build fast APIs with Rust".into()),
                    tags: Some(vec!["rust".into(), "framework".into()]),
                    author: Some("Ironic Team".into()),
                    publish: Some(true),
                    category_ids: Some(vec![fw_cat.id, tut_cat.id]),
                });

                let _ = self.create(CreateBlogDto {
                    title: "Why Rust for APIs".into(),
                    content: "Memory safety without garbage collection makes Rust ideal for high-throughput backend services.".into(),
                    excerpt: Some("Rust's performance for backend".into()),
                    tags: Some(vec!["rust".into()]),
                    author: Some("Jane Developer".into()),
                    publish: Some(true),
                    category_ids: Some(vec![rust_cat.id]),
                });

                tracing::info!("seeded 2 blog posts and 3 categories");
            }

            Ok(())
        })
    }
}

impl BlogService {
    pub fn list(&self, filter: &BlogFilterDto) -> Result<Vec<BlogPost>, HttpError> {
        self.blog_repo.list(filter)
    }

    pub fn find(&self, id: Uuid) -> Result<BlogPost, HttpError> {
        self.blog_repo.find(id)?.ok_or_else(|| {
            HttpError::not_found("POST_NOT_FOUND", format!("Blog post {id} not found"))
        })
    }

    pub fn find_by_slug(&self, slug: &str) -> Result<BlogPost, HttpError> {
        self.blog_repo.find_by_slug(slug)?.ok_or_else(|| {
            HttpError::not_found(
                "POST_NOT_FOUND",
                format!("Blog post with slug '{slug}' not found"),
            )
        })
    }

    pub fn create(&self, dto: CreateBlogDto) -> Result<BlogPost, HttpError> {
        if self.blog_repo.find_by_slug(&slugify(&dto.title))?.is_some() {
            return Err(HttpError::bad_request(
                "SLUG_CONFLICT",
                "A post with this title already exists",
            ));
        }

        let now = Utc::now();
        let post = BlogPost {
            id: Uuid::new_v4(),
            slug: slugify(&dto.title),
            title: dto.title,
            content: dto.content,
            excerpt: dto.excerpt,
            tags: dto.tags.unwrap_or_default(),
            published: dto.publish.unwrap_or(false),
            author: dto.author.unwrap_or_else(|| "Anonymous".into()),
            category_ids: dto.category_ids.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        self.blog_repo.create(post)
    }

    pub fn update(&self, id: Uuid, dto: UpdateBlogDto) -> Result<BlogPost, HttpError> {
        let current = self.find(id)?;

        let title = dto.title.clone().unwrap_or_else(|| current.title.clone());
        if title != current.title && self.blog_repo.find_by_slug(&slugify(&title))?.is_some() {
            return Err(HttpError::bad_request(
                "SLUG_CONFLICT",
                "A post with this title already exists",
            ));
        }

        let post = BlogPost {
            id,
            slug: slugify(&title),
            title,
            content: dto.content.unwrap_or_else(|| current.content.clone()),
            excerpt: dto.excerpt.unwrap_or_else(|| current.excerpt.clone()),
            tags: dto.tags.unwrap_or_else(|| current.tags.clone()),
            published: dto.published.unwrap_or(current.published),
            author: current.author.clone(),
            category_ids: dto
                .category_ids
                .unwrap_or_else(|| current.category_ids.clone()),
            created_at: current.created_at,
            updated_at: Utc::now(),
        };

        self.blog_repo.update(post)
    }

    pub fn delete(&self, id: Uuid) -> Result<(), HttpError> {
        let deleted = self.blog_repo.delete(id)?;
        if deleted {
            tracing::info!(post_id = %id, "blog post deleted");
            Ok(())
        } else {
            Err(HttpError::not_found(
                "POST_NOT_FOUND",
                format!("Blog post {id} not found"),
            ))
        }
    }

    pub fn publish(&self, id: Uuid) -> Result<BlogPost, HttpError> {
        let post = self.find(id)?;
        if post.published {
            return Err(HttpError::bad_request(
                "ALREADY_PUBLISHED",
                "Post is already published",
            ));
        }
        self.blog_repo.update(BlogPost {
            published: true,
            updated_at: Utc::now(),
            ..post
        })
    }

    pub fn unpublish(&self, id: Uuid) -> Result<BlogPost, HttpError> {
        let post = self.find(id)?;
        if !post.published {
            return Err(HttpError::bad_request(
                "ALREADY_DRAFT",
                "Post is already a draft",
            ));
        }
        self.blog_repo.update(BlogPost {
            published: false,
            updated_at: Utc::now(),
            ..post
        })
    }

    pub fn add_category(&self, id: Uuid, category_id: Uuid) -> Result<BlogPost, HttpError> {
        let _cat = self.category_repo.find(category_id)?.ok_or_else(|| {
            HttpError::not_found(
                "CATEGORY_NOT_FOUND",
                format!("Category {category_id} not found"),
            )
        })?;

        let post = self.find(id)?;
        if post.category_ids.contains(&category_id) {
            return Err(HttpError::bad_request(
                "DUPLICATE_CATEGORY",
                "Post already has this category",
            ));
        }

        let mut ids = post.category_ids;
        ids.push(category_id);
        self.blog_repo.update_category_ids(id, &ids)
    }

    pub fn remove_category(&self, id: Uuid, category_id: Uuid) -> Result<BlogPost, HttpError> {
        let post = self.find(id)?;
        let ids: Vec<Uuid> = post
            .category_ids
            .into_iter()
            .filter(|cid| *cid != category_id)
            .collect();
        self.blog_repo.update_category_ids(id, &ids)
    }

    pub fn stats(&self) -> Result<BlogStats, HttpError> {
        self.blog_repo.stats()
    }

    pub fn categories(&self) -> Result<Vec<Category>, HttpError> {
        self.category_repo.list()
    }

    pub fn create_category(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<Category, HttpError> {
        if self.category_repo.find_by_slug(&slugify(&name))?.is_some() {
            return Err(HttpError::bad_request(
                "CATEGORY_EXISTS",
                format!("Category '{name}' already exists"),
            ));
        }

        let category = Category {
            id: Uuid::new_v4(),
            slug: slugify(&name),
            name,
            description,
        };
        self.category_repo.create(category)
    }

    pub fn delete_category(&self, id: Uuid) -> Result<(), HttpError> {
        let deleted = self.category_repo.delete(id)?;
        if deleted {
            Ok(())
        } else {
            Err(HttpError::not_found(
                "CATEGORY_NOT_FOUND",
                format!("Category {id} not found"),
            ))
        }
    }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}
