mod app;
mod modules;
mod welcome;

use std::env;
use std::time::Duration;

use ironic::AxumAdapter;
use ironic::metrics::{MetricsConfig, MetricsLayer};
use ironic::prelude::*;

use app::AppModule;

#[ironic::main]
async fn main() {
    dotenvy::dotenv().ok();
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("{}:{}", host, port);

    // Production middleware stack (executed top-to-bottom):
    // Metrics → Compression → Body Limit → Timeout
    //
    // Security middleware (CORS, rate limiting, headers, CSRF) can be
    // applied per-controller or per-route:
    //   #[guard(RoleGuard::new(&["admin"]))]
    //   ControllerDefinition::new(...).middleware(CorsMiddleware::new(cors))

    let application = Application::builder()
        .module(AppModule::definition())
        .platform(
            AxumAdapter::new()
                .compression()
                .request_body_limit(5 * 1024 * 1024)  // 5MB
                .request_timeout(Duration::from_secs(30))
                .configure_router(|r| {
                    r.layer(MetricsLayer::new(MetricsConfig::default()))
                }),
        )
        .build()
        .await
        .expect("application must initialise");

    println!("🚀 basic-api → http://{} (ironic v0.3.5)", addr);

    application
        .listen(&addr)
        .await
        .expect("application server failed");
}
