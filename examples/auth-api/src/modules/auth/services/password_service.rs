use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use ironic::prelude::*;

#[derive(Injectable)]
pub struct PasswordService;

impl PasswordService {
    pub fn hash(&self, password: &str) -> Result<String, HttpError> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                HttpError::internal(
                    ironic::error_codes::codes::INTERNAL_HASH_ERROR,
                    e.to_string(),
                )
            })?;
        Ok(hash.to_string())
    }

    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, HttpError> {
        let parsed = PasswordHash::new(hash).map_err(|e| {
            HttpError::internal(
                ironic::error_codes::codes::INTERNAL_HASH_ERROR,
                e.to_string(),
            )
        })?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    }
}
