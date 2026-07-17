use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBlogDto {
    #[garde(length(min = 1, max = 200))]
    pub title: String,

    #[garde(length(min = 1))]
    pub content: String,

    #[garde(skip)]
    pub excerpt: Option<String>,

    #[garde(skip)]
    pub tags: Option<Vec<String>>,

    #[garde(skip)]
    pub author: Option<String>,

    #[garde(skip)]
    pub publish: Option<bool>,

    #[garde(skip)]
    pub category_ids: Option<Vec<uuid::Uuid>>,
}
