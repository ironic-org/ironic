#![allow(unused_imports)]

use ironic::{HttpError, Injectable, get, routes};

#[derive(Injectable)]
struct Controller;

#[routes]
impl Controller {
    #[get("/")]
    fn invalid(&self) -> Result<(), HttpError> {
        Ok(())
    }
}

fn main() {}
