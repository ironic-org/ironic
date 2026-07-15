use super::password_service::PasswordService;
use crate::modules::auth::dto::{LoginDto, RefreshDto, RegisterDto, TokenResponse};
use crate::modules::auth::entities::role::Role;
use crate::modules::auth::entities::user::User;
use ironic::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Injectable)]
pub struct AuthService {
    pub password: Arc<PasswordService>,
}

static USERS: std::sync::LazyLock<Mutex<HashMap<u64, User>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".into())
}

impl AuthService {
    pub fn register(&self, dto: RegisterDto) -> Result<User, HttpError> {
        let mut users = USERS.lock().unwrap();
        if users.values().any(|u| u.email == dto.email) {
            return Err(HttpError::bad_request(
                ironic::error_codes::codes::AUTH_EMAIL_EXISTS,
                "Email already registered",
            ));
        }
        let hash = self.password.hash(&dto.password)?;
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let user = User {
            id,
            email: dto.email,
            password_hash: hash,
            name: dto.name,
            role: Role::User,
            provider: "email".into(),
            created_at: "now".into(),
        };
        users.insert(id, user.clone());
        Ok(user)
    }

    pub fn login(&self, dto: LoginDto) -> Result<TokenResponse, HttpError> {
        let users = USERS.lock().unwrap();
        let user = users
            .values()
            .find(|u| u.email == dto.email)
            .ok_or_else(|| {
                HttpError::unauthorized(
                    ironic::error_codes::codes::AUTH_INVALID_CREDENTIALS,
                    "Invalid email or password",
                )
            })?;
        if !self.password.verify(&dto.password, &user.password_hash)? {
            return Err(HttpError::unauthorized(
                ironic::error_codes::codes::AUTH_INVALID_CREDENTIALS,
                "Invalid email or password",
            ));
        }
        self.issue_tokens(user)
    }

    pub fn refresh(&self, dto: RefreshDto) -> Result<TokenResponse, HttpError> {
        let token_data = decode::<Claims>(
            &dto.refresh_token,
            &DecodingKey::from_secret(jwt_secret().as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| {
            HttpError::unauthorized(
                ironic::error_codes::codes::AUTH_INVALID_TOKEN,
                "Invalid refresh token",
            )
        })?;
        let users = USERS.lock().unwrap();
        let id: u64 = token_data.claims.sub.parse().unwrap_or(0);
        let user = users.get(&id).ok_or_else(|| {
            HttpError::unauthorized(
                ironic::error_codes::codes::AUTH_INVALID_TOKEN,
                "User not found",
            )
        })?;
        self.issue_tokens(user)
    }

    pub fn me(&self, user_id: u64) -> Result<User, HttpError> {
        USERS.lock().unwrap().get(&user_id).cloned().ok_or_else(|| {
            HttpError::not_found(ironic::error_codes::codes::NOT_FOUND_USER, "User not found")
        })
    }

    fn issue_tokens(&self, user: &User) -> Result<TokenResponse, HttpError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let access = Claims {
            sub: user.id.to_string(),
            role: user.role.as_str().into(),
            exp: now + 900,
            iat: now,
        };
        let refresh = Claims {
            sub: user.id.to_string(),
            role: user.role.as_str().into(),
            exp: now + 604800,
            iat: now,
        };
        let at = encode(
            &Header::default(),
            &access,
            &EncodingKey::from_secret(jwt_secret().as_bytes()),
        )
        .map_err(|e| {
            HttpError::internal(
                ironic::error_codes::codes::INTERNAL_JWT_ERROR,
                e.to_string(),
            )
        })?;
        let rt = encode(
            &Header::default(),
            &refresh,
            &EncodingKey::from_secret(jwt_secret().as_bytes()),
        )
        .map_err(|e| {
            HttpError::internal(
                ironic::error_codes::codes::INTERNAL_JWT_ERROR,
                e.to_string(),
            )
        })?;
        Ok(TokenResponse {
            access_token: at,
            refresh_token: rt,
            expires_in: 900,
        })
    }

    pub fn verify_token(token: &str) -> Result<Claims, HttpError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret().as_bytes()),
            &Validation::default(),
        )
        .map(|d| d.claims)
        .map_err(|_| {
            HttpError::unauthorized(
                ironic::error_codes::codes::AUTH_INVALID_TOKEN,
                "Invalid or expired token",
            )
        })
    }
}
