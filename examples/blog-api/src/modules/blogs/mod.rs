use ironic::prelude::*;

// ── BlogsModule ──────────────────────────────────────────────────
// Demonstrates: providers, controllers, exports, lifecycle_init
// BlogService seeds data via OnModuleInit on first startup.

pub mod controller;
pub mod dto;
pub mod entities;
pub mod repositories;
pub mod services;

#[cfg(test)]
mod tests;

pub use controller::{BlogsController, CategoriesController};
pub use repositories::{BlogRepository, CategoryRepository};
pub use services::BlogService;

#[derive(Module)]
#[module(
    providers = [BlogRepository, CategoryRepository, BlogService],
    controllers = [BlogsController, CategoriesController],
    exports = [BlogService],
    lifecycle_init = [BlogService],
)]
pub struct BlogsModule;
