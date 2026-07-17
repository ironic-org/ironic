use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;
use ironic::prelude::*;
use uuid::Uuid;

use crate::modules::blogs::dto::BlogFilterDto;
use crate::modules::blogs::entities::BlogPost;

static BLOG_POSTS: std::sync::LazyLock<Mutex<HashMap<Uuid, BlogPost>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Injectable)]
pub struct BlogRepository;

impl BlogRepository {
    pub fn list(&self, filter: &BlogFilterDto) -> Result<Vec<BlogPost>, HttpError> {
        let posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        let mut result: Vec<BlogPost> = posts.values().cloned().collect();

        if let Some(published) = filter.published {
            result.retain(|p| p.published == published);
        }
        if let Some(ref author) = filter.author {
            result.retain(|p| p.author == *author);
        }
        if let Some(ref tag) = filter.tag {
            result.retain(|p| p.tags.iter().any(|t| t == tag));
        }
        if let Some(category_id) = filter.category_id {
            result.retain(|p| p.category_ids.contains(&category_id));
        }
        if let Some(ref query) = filter.search {
            let q = query.to_lowercase();
            result.retain(|p| {
                p.title.to_lowercase().contains(&q)
                    || p.content.to_lowercase().contains(&q)
                    || p.excerpt.as_deref().is_some_and(|e| e.to_lowercase().contains(&q))
            });
        }

        result.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        Ok(result)
    }

    pub fn find(&self, id: Uuid) -> Result<Option<BlogPost>, HttpError> {
        let posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        Ok(posts.get(&id).cloned())
    }

    pub fn find_by_slug(&self, slug: &str) -> Result<Option<BlogPost>, HttpError> {
        let posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        Ok(posts.values().find(|p| p.slug == slug).cloned())
    }

    pub fn create(&self, post: BlogPost) -> Result<BlogPost, HttpError> {
        let mut posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        posts.insert(post.id, post.clone());
        Ok(post)
    }

    pub fn update(&self, post: BlogPost) -> Result<BlogPost, HttpError> {
        let mut posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        posts.insert(post.id, post.clone());
        Ok(post)
    }

    pub fn delete(&self, id: Uuid) -> Result<bool, HttpError> {
        let mut posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        Ok(posts.remove(&id).is_some())
    }

    pub fn update_category_ids(
        &self,
        id: Uuid,
        category_ids: &[Uuid],
    ) -> Result<BlogPost, HttpError> {
        let mut posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        let post = posts.get_mut(&id).ok_or_else(|| {
            HttpError::not_found("POST_NOT_FOUND", format!("Post {id} not found"))
        })?;
        post.category_ids = category_ids.to_vec();
        post.updated_at = Utc::now();
        Ok(post.clone())
    }

    pub fn stats(&self) -> Result<BlogStats, HttpError> {
        let posts = BLOG_POSTS.lock().map_err(|e| {
            HttpError::internal("LOCK_ERROR", e.to_string())
        })?;
        let total = posts.len() as u64;
        let published = posts.values().filter(|p| p.published).count() as u64;
        let draft = total - published;
        let total_words: usize = posts
            .values()
            .map(|p| p.content.split_whitespace().count())
            .sum();
        Ok(BlogStats {
            total,
            published,
            draft,
            total_words: total_words as u64,
            unique_tags: {
                let mut tags = std::collections::HashSet::new();
                for p in posts.values() {
                    for t in &p.tags {
                        tags.insert(t.clone());
                    }
                }
                tags.len() as u64
            },
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogStats {
    pub total: u64,
    pub published: u64,
    pub draft: u64,
    pub total_words: u64,
    pub unique_tags: u64,
}
