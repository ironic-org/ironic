use ironic::prelude::*;

pub mod controller;
pub mod decorators;
pub mod dto;
pub mod entities;
pub mod guards;
pub mod services;

#[cfg(test)]
mod tests;

pub use controller::AuthController;
#[allow(unused_imports)]
pub use guards::AuthGuard;
pub use services::auth_service::AuthService;
pub use services::password_service::PasswordService;

#[derive(Module)]
#[module(
    providers = [AuthService, PasswordService],
    controllers = [AuthController],
    exports = [AuthService],
)]
pub struct AuthModule;
