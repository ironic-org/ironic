use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterDto {
    #[garde(length(min = 5, max = 254))]
    pub email: String,
    #[garde(length(min = 8, max = 128))]
    pub password: String,
    #[garde(length(min = 1, max = 256))]
    pub name: String,
}
