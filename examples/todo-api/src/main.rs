mod app;
mod modules;

use ironic::{AxumAdapter, prelude::*};

use app::AppModule;

#[ironic::main]
async fn main() {
    let application = FrameworkApplication::builder()
        .module(AppModule::definition())
        .platform(AxumAdapter::new())
        .build()
        .await
        .expect("application must initialize");

    application
        .listen("127.0.0.1:3000")
        .await
        .expect("application server failed");
}
