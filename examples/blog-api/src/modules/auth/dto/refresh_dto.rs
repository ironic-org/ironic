use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, OpenApiSchema)]
pub struct RefreshDto {
    pub refresh_token: String,
}
