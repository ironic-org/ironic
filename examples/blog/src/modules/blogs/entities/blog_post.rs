use ironic::OpenApiSchema;
use ironic::json::{Value, json};
use ironic::time::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub tags: Vec<String>,
    pub published: bool,
    pub author: String,
    pub category_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OpenApiSchema for BlogPost {
    fn openapi_schema() -> Value {
        json!({
            "type": "object",
            "title": "BlogPost",
            "properties": {
                "id": { "type": "string", "format": "uuid" },
                "title": { "type": "string" },
                "slug": { "type": "string" },
                "content": { "type": "string" },
                "excerpt": { "type": "string", "nullable": true },
                "tags": { "type": "array", "items": { "type": "string" } },
                "published": { "type": "boolean" },
                "author": { "type": "string" },
                "category_ids": { "type": "array", "items": { "type": "string", "format": "uuid" } },
                "created_at": { "type": "string", "format": "date-time" },
                "updated_at": { "type": "string", "format": "date-time" }
            },
            "required": ["id", "title", "slug", "content", "tags", "published", "author", "category_ids", "created_at", "updated_at"]
        })
    }
}
