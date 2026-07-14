use crate::modules::todos::entities::Priority;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodosDto {
    pub title: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<Priority>,
}
