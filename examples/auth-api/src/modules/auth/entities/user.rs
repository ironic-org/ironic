use super::role::Role;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub role: Role,
    pub provider: String,
    pub created_at: String,
}

impl User {
    /// Returns a safe view without the password hash.
    #[must_use]
    pub fn public_view(&self) -> PublicUser {
        PublicUser {
            id: self.id,
            email: self.email.clone(),
            name: self.name.clone(),
            role: self.role,
            provider: self.provider.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicUser {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub role: Role,
    pub provider: String,
}
