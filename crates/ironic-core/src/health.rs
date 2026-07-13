use std::sync::Arc;

use rustframe_di::{ProviderDefinition, Scope};
use rustframe_http::{
    ControllerDefinition, HttpError, HttpMethod, Json, RouteDefinition, handler_fn,
};
use serde::Serialize;

use crate::{Module, ModuleDefinition};

/// Imports the built-in `GET /health` readiness endpoint.
pub struct HealthModule;

impl Module for HealthModule {
    fn definition() -> ModuleDefinition {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(HealthController)
        });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "health",
            handler_fn(
                |_controller: Arc<HealthController>, _arguments| async move {
                    Ok::<_, HttpError>(Json(HealthStatus { status: "ok" }))
                },
            ),
        )
        .expect("the built-in health route is valid");
        let controller = ControllerDefinition::new::<HealthController>("/health", provider)
            .expect("the built-in health controller path is valid")
            .route(route);
        ModuleDefinition::builder::<Self>()
            .controller(controller)
            .build()
    }
}

struct HealthController;

/// The stable successful health response body.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct HealthStatus {
    /// Service readiness state.
    pub status: &'static str,
}
