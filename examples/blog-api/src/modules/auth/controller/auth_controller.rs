use std::sync::Arc;

use ironic::prelude::*;

use crate::modules::auth::dto::{LoginDto, RefreshDto, TokenResponse};
use crate::modules::auth::services::AuthService;

#[controller("/api/auth")]
#[derive(Injectable)]
pub struct AuthController {
    auth: Arc<AuthService>,
}

#[routes]
impl AuthController {
    #[api(summary = "Login", tag = "Auth")]
    #[body(json = LoginDto)]
    #[resp(200, "Authentication successful", json = TokenResponse)]
    #[resp(401, "Invalid credentials")]
    #[post("/login")]
    async fn login(&self, #[body] dto: LoginDto) -> Result<Json<TokenResponse>, HttpError> {
        let tokens = self
            .auth
            .login(&dto.username, &dto.password)
            .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
        Ok(Json(tokens))
    }

    #[api(summary = "Refresh token", tag = "Auth")]
    #[body(json = RefreshDto)]
    #[resp(200, "Token refreshed", json = TokenResponse)]
    #[resp(400, "Missing refresh token")]
    #[post("/refresh")]
    async fn refresh(&self, #[body] dto: RefreshDto) -> Result<Json<TokenResponse>, HttpError> {
        let tokens = self.auth.refresh(&dto.refresh_token)?;
        Ok(Json(tokens))
    }

    #[api(summary = "Get current user", tag = "Auth", security = "bearer")]
    #[resp(200, "Current user info")]
    #[resp(401, "Invalid or expired token")]
    #[get("/me")]
    async fn me(
        &self,
        #[header("authorization")] auth_header: String,
    ) -> Result<Json<Value>, HttpError> {
        let token = auth_header.strip_prefix("Bearer ").unwrap_or(&auth_header);
        let claims = self.auth.validate(token)?;
        Ok(Json(ironic::json::json!({
            "username": claims.username,
            "exp": claims.exp,
        })))
    }
}
