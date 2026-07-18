use ironic::OpenApiSchema;
use ironic::json::{Value, json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlogFilterDto {
    pub published: Option<bool>,
    pub author: Option<String>,
    pub tag: Option<String>,
    pub category_id: Option<uuid::Uuid>,
    pub search: Option<String>,
}

impl OpenApiSchema for BlogFilterDto {
    fn openapi_schema() -> Value {
        json!({
            "type": "object",
            "title": "BlogFilterDto",
            "properties": {
                "published": { "type": "boolean", "nullable": true },
                "author": { "type": "string", "nullable": true },
                "tag": { "type": "string", "nullable": true },
                "category_id": { "type": "string", "format": "uuid", "nullable": true },
                "search": { "type": "string", "nullable": true }
            },
            "required": []
        })
    }
}
