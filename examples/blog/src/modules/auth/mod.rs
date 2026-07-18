// ── AuthModule ──────────────────────────────────────────────────
// Demonstrates: providers, controllers, exports, guards
// JwtGuard validates Bearer tokens on protected routes.

pub mod controller;
pub mod dto;
pub mod guards;
pub mod services;

use ironic::prelude::*;

use crate::modules::auth::services::AuthService;

#[derive(Module)]
#[module(
    providers = [AuthService],
    controllers = [controller::AuthController],
    exports = [AuthService],
)]
pub struct AuthModule;
