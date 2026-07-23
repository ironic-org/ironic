use ironic::prelude::*;
pub mod controller;
pub mod services;
pub mod dto;
pub mod entities;
#[derive(Module)]
#[module()]
pub struct MyCtrlModule;
pub use controller::MyCtrlController;
