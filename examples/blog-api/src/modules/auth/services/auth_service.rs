use ironic::time::Utc;
use ironic::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::modules::auth::dto::TokenResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

#[derive(Injectable)]
pub struct AuthService;

impl AuthService {
    fn secret() -> String {
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "ironic-dev-secret".into())
    }

    pub fn login(&self, username: &str, password: &str) -> Result<TokenResponse, HttpError> {
        if username != "admin" || password != "ironic" {
            return Err(HttpError::unauthorized(
                "INVALID_CREDENTIALS",
                "Invalid username or password",
            ));
        }
        self.generate_tokens(username)
    }

    pub fn refresh(&self, refresh_token: &str) -> Result<TokenResponse, HttpError> {
        let claims = self.validate(refresh_token)?;
        self.generate_tokens(&claims.username)
    }

    pub fn validate(&self, token: &str) -> Result<Claims, HttpError> {
        let secret = Self::secret();
        let key = DecodingKey::from_secret(secret.as_bytes());
        let token_data = decode::<Claims>(token, &key, &Validation::default())
            .map_err(|e| HttpError::unauthorized("INVALID_TOKEN", format!("{e}")))?;
        Ok(token_data.claims)
    }

    fn generate_tokens(&self, username: &str) -> Result<TokenResponse, HttpError> {
        let secret = Self::secret();
        let now = Utc::now();
        let access_claims = Claims {
            sub: username.to_string(),
            username: username.to_string(),
            exp: (now + ironic::time::Duration::hours(1)).timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
        };
        let refresh_claims = Claims {
            sub: username.to_string(),
            username: username.to_string(),
            exp: (now + ironic::time::Duration::days(7)).timestamp() as usize,
            iat: now.timestamp() as usize,
            jti: uuid::Uuid::new_v4().to_string(),
        };

        let encode = |claims: &Claims| {
            encode(
                &Header::default(),
                claims,
                &EncodingKey::from_secret(secret.as_bytes()),
            )
            .map_err(|e| HttpError::internal("JWT_ENCODE_ERROR", format!("{e}")))
        };

        Ok(TokenResponse {
            access_token: encode(&access_claims)?,
            refresh_token: encode(&refresh_claims)?,
            token_type: "Bearer".into(),
            expires_in: 3600,
        })
    }
}
