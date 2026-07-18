// ── StatsModule ────────────────────────────────────────────────
// Demonstrates: cross-module DI via imports
// StatsService receives Arc<BlogService> automatically.

use ironic::prelude::*;

pub mod controller;
pub mod services;

pub use controller::StatsController;
pub use services::StatsService;

#[derive(Module)]
#[module(
    imports = [crate::modules::blogs::BlogsModule],
    providers = [StatsService],
    controllers = [StatsController],
)]
pub struct StatsModule;
