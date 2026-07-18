mod app;
mod modules;
mod platform;
mod welcome;

use std::time::Duration;

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::prelude::*;

use ironic::{OpenApiAxumExt, OpenApiConfig, SecurityScheme};
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

    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_ansi(false).with_writer(non_blocking))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = platform::config::server_address();
    let cors_origins: Vec<String> = std::env::var("CORS_ORIGINS")
        .ok()
        .and_then(|v| ironic::json::from_str(&v).ok())
        .unwrap_or_default();
    let rate_limit_max: u64 = std::env::var("RATE_LIMIT_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let application = Application::builder()
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
                .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
                .with_openapi(
                    OpenApiConfig::new("Blog API", "0.1.0")
                        .description("Blog API with CRUD endpoints")
                        .security_scheme(
                            "bearer",
                            SecurityScheme::HttpBearer {
                                bearer_format: Some("JWT".into()),
                            },
                        ),
                )
                .swagger_ui("/docs"),
        )
        .build()
        .await
        .expect("application must initialise");

    ironic::logging::log::info!(
        "blog-api → http://{} (ironic v{})",
        addr,
        env!("CARGO_PKG_VERSION")
    );

    application
        .listen(&addr)
        .await
        .expect("application server failed");
}
