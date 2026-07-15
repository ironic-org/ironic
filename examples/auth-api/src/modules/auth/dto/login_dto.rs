use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginDto {
    #[garde(length(min = 1))]
    pub email: String,
    #[garde(length(min = 1))]
    pub password: String,
}
