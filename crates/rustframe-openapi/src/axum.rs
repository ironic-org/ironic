use std::{net::SocketAddr, sync::Arc};

use axum::{Router, http::header, response::Html, routing::get};
use rustframe_http::CompiledHttpApplication;
use rustframe_platform::{
    HttpPlatformAdapter, HttpPlatformApplication, PlatformFuture, Shutdown, ShutdownSignal,
};
use rustframe_platform_axum::{AxumAdapter, AxumApplication, AxumPlatformError};

use crate::{OpenApiConfig, OpenApiDocument, OpenApiError, document::validate_absolute_path};

/// `OpenAPI` generation or delegated Axum platform failure.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum OpenApiAxumError {
    /// Document or endpoint configuration is invalid.
    #[error(transparent)]
    OpenApi(#[from] OpenApiError),
    /// The underlying Axum platform failed.
    #[error(transparent)]
    Axum(#[from] AxumPlatformError),
}

/// Axum adapter wrapper that serves an `OpenAPI` document and optional Swagger UI.
pub struct OpenApiAxumAdapter {
    inner: AxumAdapter,
    config: OpenApiConfig,
    swagger_ui_path: Option<String>,
}

impl OpenApiAxumAdapter {
    /// Serves Swagger UI at `path` using the generated JSON endpoint.
    #[must_use]
    pub fn swagger_ui(mut self, path: impl Into<String>) -> Self {
        self.swagger_ui_path = Some(path.into());
        self
    }
}

/// Adds `OpenAPI` generation to the standard Axum adapter.
pub trait OpenApiAxumExt {
    /// Wraps this adapter with automatic `OpenAPI` route discovery.
    #[must_use]
    fn with_openapi(self, config: OpenApiConfig) -> OpenApiAxumAdapter;
}

impl OpenApiAxumExt for AxumAdapter {
    fn with_openapi(self, config: OpenApiConfig) -> OpenApiAxumAdapter {
        OpenApiAxumAdapter {
            inner: self,
            config,
            swagger_ui_path: None,
        }
    }
}

impl HttpPlatformAdapter for OpenApiAxumAdapter {
    type Application = OpenApiAxumApplication;
    type Error = OpenApiAxumError;

    fn build(
        self,
        application: Arc<CompiledHttpApplication>,
    ) -> Result<Self::Application, Self::Error> {
        self.config.validate()?;
        if let Some(path) = &self.swagger_ui_path {
            validate_absolute_path(path)?;
        }
        ensure_endpoint_available(&application, self.config.json_path_value())?;
        if let Some(path) = &self.swagger_ui_path {
            if path == self.config.json_path_value() {
                return Err(OpenApiError::EndpointConflict { path: path.clone() }.into());
            }
            ensure_endpoint_available(&application, path)?;
        }
        let document = OpenApiDocument::from_application(&application, &self.config)?;
        let json_path = self.config.json_path_value().to_owned();
        let title = self.config.title().to_owned();
        let swagger_ui_path = self.swagger_ui_path;
        let native = self.inner.build(application)?.map_router(move |router| {
            openapi_router(
                router,
                document.as_value(),
                &json_path,
                swagger_ui_path,
                &title,
            )
        });
        Ok(OpenApiAxumApplication { inner: native })
    }
}

fn ensure_endpoint_available(
    application: &CompiledHttpApplication,
    path: &str,
) -> Result<(), OpenApiError> {
    if application
        .routes()
        .iter()
        .any(|route| route.method() == axum::http::Method::GET && route.path() == path)
    {
        Err(OpenApiError::EndpointConflict {
            path: path.to_owned(),
        })
    } else {
        Ok(())
    }
}

/// Built Axum application with `OpenAPI` endpoints installed.
pub struct OpenApiAxumApplication {
    inner: AxumApplication,
}

impl OpenApiAxumApplication {
    /// Returns the native Axum router.
    pub fn router(&self) -> &Router {
        self.inner.router()
    }

    /// Consumes the application and returns the native Axum router.
    pub fn into_router(self) -> Router {
        self.inner.into_router()
    }
}

impl HttpPlatformApplication for OpenApiAxumApplication {
    type Error = OpenApiAxumError;

    fn listen(
        self,
        address: SocketAddr,
        shutdown: Shutdown,
    ) -> PlatformFuture<Result<ShutdownSignal, Self::Error>> {
        Box::pin(async move {
            self.inner
                .listen(address, shutdown)
                .await
                .map_err(OpenApiAxumError::Axum)
        })
    }
}

fn openapi_router(
    router: Router,
    document: &serde_json::Value,
    json_path: &str,
    swagger_ui_path: Option<String>,
    title: &str,
) -> Router {
    let document =
        Arc::new(serde_json::to_string(document).expect("serializing a JSON value cannot fail"));
    let mut router = router.route(
        json_path,
        get(move || {
            let document = Arc::clone(&document);
            async move {
                (
                    [(header::CONTENT_TYPE, "application/json")],
                    document.as_str().to_owned(),
                )
            }
        }),
    );
    if let Some(path) = swagger_ui_path {
        let html = swagger_html(title, json_path);
        router = router.route(&path, get(move || async move { Html(html.clone()) }));
    }
    router
}

fn swagger_html(title: &str, json_path: &str) -> String {
    let title = escape_html(title);
    let json_path = serde_json::to_string(json_path).expect("serializing a string cannot fail");
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} — Swagger UI</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>SwaggerUIBundle({{url:{json_path},dom_id:'#swagger-ui',deepLinking:true}});</script>
</body>
</html>"#
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
