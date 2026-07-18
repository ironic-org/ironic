use garde::Validate;
use ironic::OpenApiSchema;
use ironic::json::{Value, json};
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

impl OpenApiSchema for CreateBlogDto {
    fn openapi_schema() -> Value {
        json!({
            "type": "object",
            "title": "CreateBlogDto",
            "properties": {
                "title": { "type": "string", "minLength": 1, "maxLength": 200 },
                "content": { "type": "string", "minLength": 1 },
                "excerpt": { "type": "string", "nullable": true },
                "tags": { "type": "array", "items": { "type": "string" }, "nullable": true },
                "author": { "type": "string", "nullable": true },
                "publish": { "type": "boolean", "nullable": true },
                "category_ids": { "type": "array", "items": { "type": "string", "format": "uuid" }, "nullable": true }
            },
            "required": ["title", "content"]
        })
    }
}
