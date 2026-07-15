use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTodoDto {
    #[garde(length(min = 1, max = 500))]
    pub title: String,

    #[garde(skip)]
    pub description: Option<String>,
}
