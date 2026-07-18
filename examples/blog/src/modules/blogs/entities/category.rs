use ironic::OpenApiSchema;
use ironic::json::{Value, json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

impl OpenApiSchema for Category {
    fn openapi_schema() -> Value {
        json!({
            "type": "object",
            "title": "Category",
            "properties": {
                "id": { "type": "string", "format": "uuid" },
                "name": { "type": "string" },
                "slug": { "type": "string" },
                "description": { "type": "string", "nullable": true }
            },
            "required": ["id", "name", "slug"]
        })
    }
}
