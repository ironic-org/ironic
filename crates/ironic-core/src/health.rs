use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};

use ironic_di::{ProviderDefinition, Scope};
use ironic_http::{
    ControllerDefinition, HttpMethod, HttpStatus, Response, RouteDefinition, handler_fn,
};
use serde::Serialize;

use crate::{Module, ModuleDefinition};

// ---------------------------------------------------------------------------
// Build info (compile-time injection via build.rs)
// ---------------------------------------------------------------------------

/// Build-time metadata about the running binary.
///
/// Captured at compile time via `build.rs` environment variables,
/// falling back to `"unknown"` when not set (e.g. during `cargo check`).
///
/// # Examples
///
/// ```rust
/// use ironic::BuildInfo;
///
/// let info = BuildInfo::capture();
/// assert!(!info.version.is_empty());
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct BuildInfo {
    /// Git commit SHA (short).
    pub git_sha: String,
    /// Build timestamp (Unix seconds or RFC 3339 when set by CI).
    pub build_timestamp: String,
    /// Rust compiler version.
    pub rust_version: String,
    /// Active Cargo feature flags at compile time.
    pub features: Vec<String>,
    /// Ironic version (semver).
    pub version: String,
}

impl BuildInfo {
    /// Captures build-time environment variables injected by `build.rs`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ironic::BuildInfo;
    ///
    /// let info = BuildInfo::capture();
    /// assert!(!info.rust_version.is_empty());
    /// ```
    #[must_use]
    pub fn capture() -> Self {
        Self {
            git_sha: option_env!("IRONIC_GIT_SHA")
                .unwrap_or("unknown")
                .to_string(),
            build_timestamp: option_env!("IRONIC_BUILD_TIMESTAMP")
                .unwrap_or("unknown")
                .to_string(),
            rust_version: option_env!("IRONIC_RUST_VERSION")
                .unwrap_or("unknown")
                .to_string(),
            features: Self::active_features(),
            version: option_env!("CARGO_PKG_VERSION")
                .unwrap_or("unknown")
                .to_string(),
        }
    }

    #[allow(clippy::vec_init_then_push)]
    fn active_features() -> Vec<String> {
        let mut f = Vec::new();
        #[cfg(feature = "auth")]
        f.push("auth".to_string());
        #[cfg(feature = "jwt")]
        f.push("jwt".to_string());
        #[cfg(feature = "oauth")]
        f.push("oauth".to_string());
        #[cfg(feature = "sessions")]
        f.push("sessions".to_string());
        #[cfg(feature = "metrics")]
        f.push("metrics".to_string());
        #[cfg(feature = "resilience")]
        f.push("resilience".to_string());
        #[cfg(feature = "resilience-ext")]
        f.push("resilience-ext".to_string());
        #[cfg(feature = "telemetry")]
        f.push("telemetry".to_string());
        #[cfg(feature = "logging")]
        f.push("logging".to_string());
        #[cfg(feature = "openapi")]
        f.push("openapi".to_string());
        #[cfg(feature = "multipart")]
        f.push("multipart".to_string());
        #[cfg(feature = "compression")]
        f.push("compression".to_string());
        #[cfg(feature = "static-files")]
        f.push("static-files".to_string());
        #[cfg(feature = "serialization")]
        f.push("serialization".to_string());
        #[cfg(feature = "validation")]
        f.push("validation".to_string());
        #[cfg(feature = "security")]
        f.push("security".to_string());
        #[cfg(feature = "hot-reload")]
        f.push("hot-reload".to_string());
        f
    }
}

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
///
/// Implement [`check_liveness`] to report whether the component's process is
/// alive (typically always `Ok`). Implement [`check_readiness`] to report
/// whether the component is ready to serve traffic (e.g. database connected).
pub trait HealthIndicator: Send + Sync {
    /// Human-readable name (e.g. `"postgres"`, `"redis"`).
    fn name(&self) -> &str;
    /// Perform a health check and return the result.
    #[deprecated(since = "0.5.0", note = "use `check_readiness` instead")]
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>>;
    /// Reports whether the component's process is alive.
    ///
    /// Default implementation returns `HealthStatus::Ok`.
    fn check_liveness(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(std::future::ready(HealthStatus::Ok))
    }
    /// Reports whether the component is ready to serve traffic.
    ///
    /// Default implementation delegates to the deprecated [`check`] method for
    /// backward compatibility.
    fn check_readiness(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        #[allow(deprecated)]
        self.check()
    }
}

// ---------------------------------------------------------------------------
// HealthStatus enum
// ---------------------------------------------------------------------------

/// The result of a single health check.
///
/// # Examples
///
/// ```rust
/// use ironic::HealthStatus;
///
/// let ok = HealthStatus::Ok;
/// let degraded = HealthStatus::Degraded { message: None };
/// let unhealthy = HealthStatus::Unhealthy { error: "db down".into() };
/// ```
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
///
/// # Examples
///
/// ```rust
/// use ironic::HealthConfig;
/// use std::time::Duration;
///
/// let config = HealthConfig::default();
/// assert_eq!(config.check_timeout, Duration::from_secs(5));
/// ```
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

#[derive(Serialize)]
struct LivenessResponse {
    status: String,
}

// ---------------------------------------------------------------------------
// HealthModule
// ---------------------------------------------------------------------------

/// Imports the composite `GET /health` readiness endpoint, plus
/// `GET /health/live` (liveness probe) and `GET /health/ready` (readiness probe).
///
/// Also imports `GET /version` returning build metadata.
///
/// Registered [`HealthIndicator`]s are checked in parallel on each request.
/// The aggregate status is `ok` (200), `degraded` (207), or `unhealthy` (503).
pub struct HealthModule;

impl Module for HealthModule {
    fn definition() -> ModuleDefinition {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(HealthController)
        });
        let version_provider =
            ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
                Ok(VersionController)
            });

        // GET /health — composite health check (backward compatible)
        let health_route = RouteDefinition::new(
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
                    Response::json(status, &response)
                },
            ),
        )
        .expect("the built-in health route is valid");

        // GET /health/live — liveness probe
        let liveness_route = RouteDefinition::new(
            HttpMethod::GET,
            "/live",
            "health_live",
            handler_fn(
                |_controller: Arc<HealthController>, _arguments| async move {
                    Response::json(
                        HttpStatus::OK,
                        &LivenessResponse {
                            status: "alive".into(),
                        },
                    )
                },
            ),
        )
        .expect("the built-in liveness route is valid");

        // GET /health/ready — readiness probe (aggregates check_readiness)
        let readiness_route = RouteDefinition::new(
            HttpMethod::GET,
            "/ready",
            "health_ready",
            handler_fn(
                |_controller: Arc<HealthController>, _arguments| async move {
                    let response = run_readiness_checks().await;
                    let status = match response.status.as_str() {
                        "unhealthy" | "degraded" => HttpStatus::SERVICE_UNAVAILABLE,
                        _ => HttpStatus::OK,
                    };
                    Response::json(status, &response)
                },
            ),
        )
        .expect("the built-in readiness route is valid");

        // GET /version — build info
        let version_route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "version",
            handler_fn(
                |_controller: Arc<VersionController>, _arguments| async move {
                    let info = BuildInfo::capture();
                    Response::json(HttpStatus::OK, &info)
                },
            ),
        )
        .expect("the built-in version route is valid");

        let health_controller = ControllerDefinition::new::<HealthController>("/health", provider)
            .expect("the built-in health controller path is valid")
            .route(health_route)
            .route(liveness_route)
            .route(readiness_route);

        let version_controller =
            ControllerDefinition::new::<VersionController>("/version", version_provider)
                .expect("the built-in version controller path is valid")
                .route(version_route);

        ModuleDefinition::builder::<Self>()
            .controller(health_controller)
            .controller(version_controller)
            .build()
    }
}

struct HealthController;

struct VersionController;

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
        #[allow(deprecated)]
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

async fn run_readiness_checks() -> HealthResponse {
    let config = load_config();
    let mut checks: HashMap<String, CheckResult> = HashMap::new();
    let mut aggregate: &str = "ok";

    let indicators: Vec<Arc<dyn HealthIndicator>> = HEALTH_INDICATORS
        .lock()
        .ok()
        .map(|list| list.clone())
        .unwrap_or_default();

    for indicator in &indicators {
        let result = tokio::time::timeout(config.check_timeout, indicator.check_readiness()).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_info_capture_has_defaults() {
        let info = BuildInfo::capture();
        assert!(!info.git_sha.is_empty());
        assert!(!info.build_timestamp.is_empty());
        assert!(!info.rust_version.is_empty());
        assert!(!info.version.is_empty());
    }

    #[test]
    fn health_config_default_timeout() {
        let config = HealthConfig::default();
        assert_eq!(config.check_timeout, Duration::from_secs(5));
    }

    #[test]
    fn health_config_custom_timeout() {
        let config = HealthConfig {
            check_timeout: Duration::from_secs(30),
        };
        assert_eq!(config.check_timeout, Duration::from_secs(30));
    }

    #[test]
    fn health_status_ok_debug_and_serialize() {
        let status = HealthStatus::Ok;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""ok""#);
    }

    #[test]
    fn health_status_degraded_with_message() {
        let status = HealthStatus::Degraded {
            message: Some("high latency".into()),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("high latency"));
    }

    #[test]
    fn health_status_degraded_without_message_omits_field() {
        let status = HealthStatus::Degraded { message: None };
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#"{"degraded":{}}"#);
    }

    #[test]
    fn health_status_unhealthy() {
        let status = HealthStatus::Unhealthy {
            error: "connection refused".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("connection refused"));
    }

    #[test]
    fn configure_overrides_default_config() {
        let original = load_config();
        let custom = HealthConfig {
            check_timeout: Duration::from_secs(15),
        };
        configure(custom.clone());
        assert_eq!(load_config().check_timeout, Duration::from_secs(15));
        configure(original);
    }

    #[test]
    fn register_adds_indicators() {
        use std::sync::Arc;

        struct DummyIndicator;
        impl HealthIndicator for DummyIndicator {
            fn name(&self) -> &'static str {
                "dummy"
            }
            #[allow(deprecated)]
            fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
                Box::pin(std::future::ready(HealthStatus::Ok))
            }
        }

        let indicator = Arc::new(DummyIndicator);
        register(indicator);

        let list = HEALTH_INDICATORS.lock().ok();
        assert!(list.is_some());
        if let Some(list) = list {
            let found = list.iter().any(|i| i.name() == "dummy");
            assert!(found);
        }
    }
}
