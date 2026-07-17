use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBlogDto {
    pub title: Option<String>,
    pub content: Option<String>,
    pub excerpt: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub published: Option<bool>,
    pub category_ids: Option<Vec<uuid::Uuid>>,
}
