use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateExampleDto {
    #[garde(length(min = 1, max = 256))]
    pub name: String,
    #[garde(skip)]
    pub description: Option<String>,
}
