use crate::modules::todos::entities::Priority;
use garde::Validate;
use serde::{Deserialize, Serialize};

fn default_priority() -> Priority {
    Priority::Medium
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTodosDto {
    #[garde(length(min = 1, max = 256))]
    pub title: String,
    #[serde(default = "default_priority")]
    #[garde(skip)]
    pub priority: Priority,
}
