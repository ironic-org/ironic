use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExampleDto {
    pub name: Option<String>,
    pub description: Option<String>,
}
