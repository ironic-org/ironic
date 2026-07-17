use ironic::prelude::*;

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
)]
pub struct BlogsModule;
