use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlogFilterDto {
    pub published: Option<bool>,
    pub author: Option<String>,
    pub tag: Option<String>,
    pub category_id: Option<uuid::Uuid>,
    pub search: Option<String>,
}
