use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, OpenApiSchema)]
pub struct LoginDto {
    pub username: String,
    pub password: String,
}
