mod app;
mod modules;
mod platform;
mod welcome;

use std::time::Duration;

use ironic::AxumAdapter;
use ironic::metrics::{MetricsConfig, MetricsLayer};
use ironic::prelude::*;
use ironic::security::{
    CorsConfig, CorsMiddleware, RateLimitMiddleware, SecurityHeadersConfig,
    SecurityHeadersMiddleware,
};

use app::AppModule;

#[ironic::main]
async fn main() {
    dotenvy::dotenv().ok();
    platform::telemetry::init_tracing();

    let pool = platform::database::build_pool().await;
    platform::database::DB_POOL.set(pool).ok();

    let addr = platform::config::server_address();
    let cors_origins = platform::config::env_json_array("CORS_ORIGINS");
    let rate_limit_max = platform::config::env_parsed("RATE_LIMIT_MAX", 100u64);

    let application = FrameworkApplication::builder()
        .module(AppModule::definition())
        .middleware(SecurityHeadersMiddleware::new(
            SecurityHeadersConfig::default(),
        ))
        .middleware(RateLimitMiddleware::new(rate_limit_max, 60))
        .middleware(CorsMiddleware::new(
            CorsConfig::new().allowed_origins(cors_origins),
        ))
        .platform(
            AxumAdapter::new()
                .compression()
                .request_body_limit(5 * 1024 * 1024)
                .request_timeout(Duration::from_secs(30))
                .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default()))),
        )
        .build()
        .await
        .expect("application must initialise");

    tracing::info!("todo-example → http://{} (ironic v0.4.0)", addr);

    application
        .listen(&addr)
        .await
        .expect("application server failed");
}
