use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub id: u64,
    pub name: String,
    pub description: String,
}
