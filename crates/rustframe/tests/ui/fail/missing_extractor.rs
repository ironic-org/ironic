#![allow(unused_imports)]

use rustframe::{HttpError, Injectable, get, routes};

#[derive(Injectable)]
struct Controller;

#[routes]
impl Controller {
    #[get("/")]
    async fn invalid(&self, value: String) -> Result<(), HttpError> {
        let _ = value;
        Ok(())
    }
}

fn main() {}
