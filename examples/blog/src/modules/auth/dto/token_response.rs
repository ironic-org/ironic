use ironic::OpenApiSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, OpenApiSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}
