use std::sync::Arc;

use ironic::prelude::*;

use crate::modules::auth::dto::{LoginDto, TokenResponse};
use crate::modules::auth::services::AuthService;

#[controller("/api/auth")]
#[derive(Injectable)]
pub struct AuthController {
    auth: Arc<AuthService>,
}

#[routes]
impl AuthController {
    #[post("/login")]
    async fn login(&self, #[body] dto: LoginDto) -> Result<Json<TokenResponse>, HttpError> {
        let tokens = self.auth.login(&dto.username, &dto.password)?;
        Ok(Json(tokens))
    }

    #[post("/refresh")]
    async fn refresh(
        &self,
        #[body] payload: Value,
    ) -> Result<Json<TokenResponse>, HttpError> {
        let refresh_token = payload["refresh_token"]
            .as_str()
            .ok_or_else(|| HttpError::bad_request("MISSING_TOKEN", "refresh_token is required"))?;
        let tokens = self.auth.refresh(refresh_token)?;
        Ok(Json(tokens))
    }

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
