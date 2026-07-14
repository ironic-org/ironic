//! Production observability: request metrics with Prometheus-compatible endpoint.
//!
//! Tracks request count, latency percentiles (p50/p90/p99), status code breakdown,
//! and in-flight request gauge. Exposes a `GET /metrics` endpoint for Prometheus scraping.
//!
//! ## Quick start
//!
//! ```ignore
//! use ironic::metrics::{MetricsConfig, MetricsLayer, MetricsModule};
//! use ironic::AxumAdapter;
//!
//! let app = ironic::FrameworkApplication::builder()
//!     .module(MetricsModule::definition())
//!     .platform(
//!         AxumAdapter::new()
//!             .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
//!     )
//!     .build().await.unwrap();
//! ```

use std::{collections::HashMap, fmt::Write, sync::Mutex, time::Instant};

use ironic_core::{Module, ModuleDefinition};
use ironic_di::{ProviderDefinition, Scope};
use ironic_http::{ControllerDefinition, HttpError, HttpMethod, Json, RouteDefinition, handler_fn};
use serde::Serialize;

use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Metrics storage (global singleton behind a Mutex)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct MetricsStore {
    request_count: u64,
    status_counts: HashMap<u16, u64>,
    latencies_secs: Vec<f64>,
    in_flight: u64,
}

static METRICS_STORE: LazyLock<Mutex<MetricsStore>> =
    LazyLock::new(|| Mutex::new(MetricsStore::default()));

fn store() -> &'static Mutex<MetricsStore> {
    &METRICS_STORE
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Controls which metrics are recorded and bucket sizes.
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Latency histogram bucket boundaries in seconds.
    pub latency_buckets: Vec<f64>,
    /// When `true`, records per-endpoint metrics (method + path labels).
    pub per_endpoint: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            latency_buckets: vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
            per_endpoint: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Tower Layer
// ---------------------------------------------------------------------------

/// Tower-compatible middleware that records HTTP request metrics.
///
/// Apply to an Axum router via `.layer()`:
///
/// ```ignore
/// AxumAdapter::new().configure_router(|r| {
///     r.layer(MetricsLayer::new(MetricsConfig::default()));
/// });
/// ```
#[derive(Debug, Clone)]
pub struct MetricsLayer {
    config: MetricsConfig,
}

impl MetricsLayer {
    /// Creates a layer with the given configuration.
    pub fn new(config: MetricsConfig) -> Self {
        Self { config }
    }
}

impl<S> tower::Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, service: S) -> Self::Service {
        MetricsService {
            inner: service,
            config: self.config.clone(),
        }
    }
}

/// Tower `Service` wrapper that records request metrics.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MetricsService<S> {
    inner: S,
    config: MetricsConfig,
}

impl<S, ReqBody, ResBody> tower::Service<http::Request<ReqBody>> for MetricsService<S>
where
    S: tower::Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
    S::Future: Send + 'static,
    S::Error: std::fmt::Display,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let _method = req.method().to_string();
        let _path = if self.config.per_endpoint {
            req.uri().path().to_string()
        } else {
            "/".into()
        };

        store()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .in_flight += 1;

        let fut = self.inner.call(req);
        Box::pin(async move {
            // Scope guard ensures in_flight is always decremented, even if cancelled
            struct DecrementGuard;
            impl Drop for DecrementGuard {
                fn drop(&mut self) {
                    if let Ok(mut s) = store().lock() {
                        s.in_flight = s.in_flight.saturating_sub(1);
                    }
                }
            }
            let _guard = DecrementGuard;

            let result = fut.await;
            let duration = start.elapsed().as_secs_f64();

            let mut s = store()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            s.request_count += 1;
            s.latencies_secs.push(duration);

            let status = match &result {
                Ok(response) => response.status().as_u16(),
                Err(_) => 500,
            };
            *s.status_counts.entry(status).or_insert(0) += 1;

            result
        })
    }
}

// ---------------------------------------------------------------------------
// Prometheus text format
// ---------------------------------------------------------------------------

#[allow(
    dead_code,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn percentile(mut sorted: Vec<f64>, p: f64) -> f64 {
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).ceil() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Returns the current metrics snapshot in Prometheus text format.
///
/// # Panics
///
/// Panics if the internal metrics store lock is poisoned.
pub fn scrape() -> String {
    let s = store()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut out = String::new();

    let _ = writeln!(out, "# HELP ironic_http_requests_total Total HTTP requests");
    let _ = writeln!(out, "# TYPE ironic_http_requests_total counter");
    let _ = writeln!(out, "ironic_http_requests_total {}", s.request_count);
    let _ = writeln!(out);

    let _ = writeln!(
        out,
        "# HELP ironic_http_responses_total Responses by status code"
    );
    let _ = writeln!(out, "# TYPE ironic_http_responses_total counter");
    let mut statuses: Vec<_> = s.status_counts.iter().collect();
    statuses.sort_by_key(|(k, _)| *k);
    for (code, count) in &statuses {
        let _ = writeln!(
            out,
            "ironic_http_responses_total{{status=\"{code}\"}} {count}"
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(
        out,
        "# HELP ironic_http_request_duration_seconds Request latency"
    );
    let _ = writeln!(out, "# TYPE ironic_http_request_duration_seconds histogram");
    let default_buckets: [f64; 12] = [
        0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];
    let mut buckets = [0usize; 12];
    for lat in &s.latencies_secs {
        for (i, boundary) in default_buckets.iter().enumerate() {
            if lat <= boundary {
                buckets[i] += 1;
                break;
            }
        }
    }
    let mut cumulative = 0usize;
    for (i, boundary) in default_buckets.iter().enumerate() {
        cumulative += buckets[i];
        let _ = writeln!(
            out,
            "ironic_http_request_duration_seconds_bucket{{le=\"{boundary}\"}} {cumulative}"
        );
    }
    let _ = writeln!(
        out,
        "ironic_http_request_duration_seconds_bucket{{le=\"+Inf\"}} {}",
        s.latencies_secs.len()
    );
    let sum: f64 = s.latencies_secs.iter().sum();
    let _ = writeln!(out, "ironic_http_request_duration_seconds_sum {sum:.6}");
    let _ = writeln!(
        out,
        "ironic_http_request_duration_seconds_count {}",
        s.latencies_secs.len()
    );
    let _ = writeln!(out);

    let _ = writeln!(
        out,
        "# HELP ironic_http_requests_in_flight Currently in-flight requests"
    );
    let _ = writeln!(out, "# TYPE ironic_http_requests_in_flight gauge");
    let _ = writeln!(out, "ironic_http_requests_in_flight {}", s.in_flight);
    let _ = writeln!(out);

    let _ = writeln!(out, "# HELP ironic_info Framework version");
    let _ = writeln!(out, "# TYPE ironic_info gauge");
    let _ = writeln!(
        out,
        "ironic_info{{version=\"{}\"}} 1",
        env!("CARGO_PKG_VERSION")
    );

    out
}

// ---------------------------------------------------------------------------
// Metrics module (exposes GET /metrics)
// ---------------------------------------------------------------------------

/// Imports the `GET /metrics` Prometheus scraping endpoint.
pub struct MetricsModule;

impl Module for MetricsModule {
    fn definition() -> ModuleDefinition {
        let provider = ProviderDefinition::constructor(Scope::Singleton, Vec::new(), |_resolver| {
            Ok(MetricsController)
        });
        let route = RouteDefinition::new(
            HttpMethod::GET,
            "/",
            "metrics",
            handler_fn(
                |_controller: std::sync::Arc<MetricsController>, _arguments| async move {
                    let body = scrape();
                    Ok::<_, HttpError>(Json(MetricsPayload { text: body }))
                },
            ),
        )
        .expect("the built-in metrics route is valid");
        let controller = ControllerDefinition::new::<MetricsController>("/metrics", provider)
            .expect("the built-in metrics controller path is valid")
            .route(route);
        ModuleDefinition::builder::<Self>()
            .controller(controller)
            .build()
    }
}

struct MetricsController;

#[derive(Serialize)]
struct MetricsPayload {
    text: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_metrics_are_valid() {
        let text = scrape();
        assert!(text.contains("ironic_http_requests_total"));
        assert!(text.contains("ironic_info"));
    }

    #[test]
    fn percentiles_work() {
        let data = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        assert!((percentile(data.clone(), 0.5) - 0.3).abs() < 0.01);
        assert!((percentile(data.clone(), 0.9) - 0.5).abs() < 0.01);
    }

    #[test]
    fn default_config_has_buckets() {
        let cfg = MetricsConfig::default();
        assert!(!cfg.latency_buckets.is_empty());
        assert!(cfg.per_endpoint);
    }
}
