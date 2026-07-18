use ironic::OpenApiSchema;
use ironic::json::{Value, json};
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

impl OpenApiSchema for UpdateBlogDto {
    fn openapi_schema() -> Value {
        json!({
            "type": "object",
            "title": "UpdateBlogDto",
            "properties": {
                "title": { "type": "string", "nullable": true },
                "content": { "type": "string", "nullable": true },
                "excerpt": { "type": "string", "nullable": true },
                "tags": { "type": "array", "items": { "type": "string" }, "nullable": true },
                "published": { "type": "boolean", "nullable": true },
                "category_ids": { "type": "array", "items": { "type": "string", "format": "uuid" }, "nullable": true }
            },
            "required": []
        })
    }
}
