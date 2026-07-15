use ironic::prelude::*;

pub mod controller;
pub mod services;
pub mod dto;
pub mod entities;

#[cfg(test)]
mod tests;

pub use controller::TodosController;
pub use services::TodoService;

#[derive(Module)]
#[module(providers = [TodoService], controllers = [TodosController])]
pub struct TodosModule;
