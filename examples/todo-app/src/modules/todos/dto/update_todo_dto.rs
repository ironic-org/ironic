use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodoDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}
