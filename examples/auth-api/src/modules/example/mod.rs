use ironic::prelude::*;

pub mod controller;
pub mod dto;
pub mod entities;
pub mod services;

#[cfg(test)]
mod tests;

pub use controller::ExampleController;
pub use services::ExampleService;

#[derive(Module)]
#[module(providers = [ExampleService], controllers = [ExampleController])]
pub struct ExampleModule;
