use std::sync::Arc;

use ironic::prelude::*;

use crate::modules::auth::dto::LoginDto;
use crate::modules::auth::dto::RefreshDto;
use crate::modules::auth::dto::TokenResponse;
use crate::modules::auth::services::AuthService;

#[controller("/api/auth")]
#[derive(Injectable)]
pub struct AuthController {
    auth: Arc<AuthService>,
}

#[routes]
impl AuthController {
    #[post("/login")]
    #[api(summary = "Login", tag = "Auth")]
    #[body(json = LoginDto)]
    #[resp(200, "Login successful", json = TokenResponse)]
    #[resp(401, "Invalid credentials")]
    async fn login(&self, #[body] dto: LoginDto) -> Result<Json<TokenResponse>, HttpError> {
        let tokens = self
            .auth
            .login(&dto.username, &dto.password)
            .exception(|e| HttpError::unauthorized("LOGIN_FAILED", e.message()))?;
        Ok(Json(tokens))
    }

    #[post("/refresh")]
    #[api(summary = "Refresh token", tag = "Auth")]
    #[body(json = RefreshDto)]
    #[resp(200, "Token refreshed", json = TokenResponse)]
    #[resp(401, "Invalid refresh token")]
    async fn refresh(&self, #[body] payload: Value) -> Result<Json<TokenResponse>, HttpError> {
        let refresh_token = payload["refresh_token"]
            .as_str()
            .ok_or_else(|| HttpError::bad_request("MISSING_TOKEN", "refresh_token is required"))?;
        let tokens = self.auth.refresh(refresh_token)?;
        Ok(Json(tokens))
    }

    #[get("/me")]
    #[api(summary = "Get current user", tag = "Auth", security = "bearerAuth")]
    #[resp(200, "Current user profile")]
    #[resp(401, "Unauthorized")]
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
