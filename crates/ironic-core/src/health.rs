use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};

use ironic_di::{ProviderDefinition, Scope};
use ironic_http::{
    ControllerDefinition, FrameworkResponse, HttpMethod, HttpStatus, RouteDefinition, handler_fn,
};
use serde::Serialize;

use crate::{Module, ModuleDefinition};

// ---------------------------------------------------------------------------
// Global registry of health checkers
// ---------------------------------------------------------------------------

static HEALTH_INDICATORS: LazyLock<Mutex<Vec<Arc<dyn HealthIndicator>>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Register a health indicator (called by integration modules at startup).
pub fn register(indicator: Arc<dyn HealthIndicator>) {
    if let Ok(mut list) = HEALTH_INDICATORS.lock() {
        list.push(indicator);
    }
}

// ---------------------------------------------------------------------------
// HealthIndicator trait
// ---------------------------------------------------------------------------

/// A component that can report its health status.
pub trait HealthIndicator: Send + Sync {
    /// Human-readable name (e.g. `"postgres"`, `"redis"`).
    fn name(&self) -> &str;
    /// Perform a health check and return the result.
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>>;
}

// ---------------------------------------------------------------------------
// HealthStatus enum
// ---------------------------------------------------------------------------

/// The result of a single health check.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    /// The component is functioning normally.
    Ok,
    /// The component is working but with degraded performance.
    Degraded {
        /// Optional human-readable explanation of the degradation.
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// The component has failed and cannot serve requests.
    Unhealthy {
        /// Description of the failure.
        error: String,
    },
}

// ---------------------------------------------------------------------------
// HealthConfig
// ---------------------------------------------------------------------------

/// Configuration for the composite health endpoint.
#[derive(Debug, Clone)]
pub struct HealthConfig {
    /// Per-check timeout (default 5s).
    pub check_timeout: Duration,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_timeout: Duration::from_secs(5),
        }
    }
}

static HEALTH_CONFIG: LazyLock<Mutex<Option<HealthConfig>>> = LazyLock::new(|| Mutex::new(None));

/// Override the default health configuration.
#[allow(unreachable_pub, dead_code)]
pub fn configure(config: HealthConfig) {
    if let Ok(mut c) = HEALTH_CONFIG.lock() {
        *c = Some(config);
    }
}

fn load_config() -> HealthConfig {
    HEALTH_CONFIG
        .lock()
        .ok()
        .and_then(|c| c.clone())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CheckResult {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    checks: HashMap<String, CheckResult>,
}

// ---------------------------------------------------------------------------
// HealthModule
// ---------------------------------------------------------------------------

/// Imports the composite `GET /health` readiness endpoint.
///
/// Registered [`HealthIndicator`]s are checked in parallel on each request.
/// The aggregate status is `ok` (200), `degraded` (207), or `unhealthy` (503).
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
                    let response = run_health_checks().await;
                    let status = match response.status.as_str() {
                        "degraded" => HttpStatus::MULTI_STATUS,
                        "unhealthy" => HttpStatus::SERVICE_UNAVAILABLE,
                        _ => HttpStatus::OK,
                    };
                    FrameworkResponse::json(status, &response)
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

async fn run_health_checks() -> HealthResponse {
    let config = load_config();
    let mut checks: HashMap<String, CheckResult> = HashMap::new();
    let mut aggregate: &str = "ok";

    let indicators: Vec<Arc<dyn HealthIndicator>> = HEALTH_INDICATORS
        .lock()
        .ok()
        .map(|list| list.clone())
        .unwrap_or_default();

    for indicator in &indicators {
        let result = tokio::time::timeout(config.check_timeout, indicator.check()).await;
        let (status, message): (String, Option<String>) = match result {
            Ok(HealthStatus::Ok) => ("ok".into(), None),
            Ok(HealthStatus::Degraded { message: msg }) => {
                if aggregate == "ok" {
                    aggregate = "degraded";
                }
                ("degraded".into(), msg)
            }
            Ok(HealthStatus::Unhealthy { error }) => {
                aggregate = "unhealthy";
                ("unhealthy".into(), Some(error))
            }
            Err(_) => {
                if aggregate == "ok" {
                    aggregate = "degraded";
                }
                (
                    "degraded".into(),
                    Some(format!(
                        "health check timed out after {:?}",
                        config.check_timeout
                    )),
                )
            }
        };
        checks.insert(
            indicator.name().to_string(),
            CheckResult { status, message },
        );
    }

    HealthResponse {
        status: aggregate.to_string(),
        checks,
    }
}
