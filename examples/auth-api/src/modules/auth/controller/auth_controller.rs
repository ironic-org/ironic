use super::super::services::AuthService;
use crate::modules::auth::dto::{LoginDto, RefreshDto, RegisterDto, TokenResponse};
use crate::modules::auth::entities::user::PublicUser;
use ironic::prelude::*;
use std::sync::Arc;

use super::super::decorators::current_user::current_user;
use super::super::guards::AuthGuard;

#[controller("/auth")]
#[derive(Injectable)]
pub struct AuthController {
    service: Arc<AuthService>,
}

#[routes]
impl AuthController {
    #[post("/register")]
    async fn register(&self, #[body] dto: RegisterDto) -> Result<Json<PublicUser>, HttpError> {
        Ok(Json(self.service.register(dto)?.public_view()))
    }

    #[post("/login")]
    async fn login(&self, #[body] dto: LoginDto) -> Result<Json<TokenResponse>, HttpError> {
        Ok(Json(self.service.login(dto)?))
    }

    #[post("/refresh")]
    async fn refresh(&self, #[body] dto: RefreshDto) -> Result<Json<TokenResponse>, HttpError> {
        Ok(Json(self.service.refresh(dto)?))
    }

    #[get("/me")]
    #[guard(AuthGuard)]
    async fn me(
        &self,
        #[custom(current_user)] user_id: u64,
    ) -> Result<Json<PublicUser>, HttpError> {
        Ok(Json(self.service.me(user_id)?.public_view()))
    }
}
