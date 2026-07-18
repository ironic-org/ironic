use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, OpenApiSchema)]
pub struct CreateCategoryDto {
    pub name: String,
    pub description: Option<String>,
}
