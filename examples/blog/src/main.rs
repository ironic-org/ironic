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
use ironic::{OpenApiAxumExt, OpenApiConfig, SecurityScheme};

use crate::modules::auth::dto::{LoginDto, TokenResponse};
use crate::modules::blogs::dto::{BlogFilterDto, CreateBlogDto, CreateCategoryDto, UpdateBlogDto};
use crate::modules::blogs::entities::{BlogPost, Category};
use app::AppModule;

#[ironic::main]
async fn main() {
    dotenvy::dotenv().ok();

    let addr = platform::config::server_address();
    let cors_origins: Vec<String> = std::env::var("CORS_ORIGINS")
        .ok()
        .and_then(|v| ironic::json::from_str(&v).ok())
        .unwrap_or_default();
    let rate_limit_max: u64 = std::env::var("RATE_LIMIT_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let openapi_config = OpenApiConfig::new("Blog API", env!("CARGO_PKG_VERSION"))
        .description("Blog API — CRUD, categories, cross-module DI, search, stats")
        .schema::<LoginDto>("LoginDto")
        .schema::<TokenResponse>("TokenResponse")
        .schema::<CreateBlogDto>("CreateBlogDto")
        .schema::<UpdateBlogDto>("UpdateBlogDto")
        .schema::<BlogFilterDto>("BlogFilterDto")
        .schema::<CreateCategoryDto>("CreateCategoryDto")
        .schema::<BlogPost>("BlogPost")
        .schema::<Category>("Category")
        .security_scheme(
            "bearerAuth",
            SecurityScheme::HttpBearer {
                bearer_format: Some("JWT".to_owned()),
            },
        );


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
                .with_openapi(openapi_config)
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
    ironic::logging::log::info!("openapi json → http://{}/openapi.json", addr);
    ironic::logging::log::info!("swagger ui  → http://{}/docs", addr);

    application
        .listen(&addr)
        .await
        .expect("application server failed");
}
