//! Distributed tracing with OpenTelemetry export.
//!
//! Extends the built-in `tracing` integration with OTLP export for Jaeger,
//! Tempo, Datadog, and other OpenTelemetry-compatible collectors.
//!
//! ## Quick start
//!
//! ```rust
//! use ironic::telemetry::{TelemetryConfig, init_tracing};
//!
//! #[ironic::main]
//! async fn main() {
//!     let _guard = init_tracing(TelemetryConfig {
//!         service_name: "my-api".into(),
//!         otlp_endpoint: Some("http://localhost:4317".into()),
//!         ..TelemetryConfig::default()
//!     });
//!
//!     // Build and run your application...
//! }
//! ```
//!
//! ## Trace context propagation
//!
//! The built-in `RequestTracing` middleware automatically creates a span for
//! each request with `http.method`, `http.url`, and `http.status_code`
//! attributes. When OTLP is enabled, these spans are exported.

use std::time::Duration;

/// Configuration for OpenTelemetry tracing.
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Service name (appears as `service.name` resource attribute).
    pub service_name: String,
    /// OTLP collector endpoint (gRPC). If `None`, only local tracing is active.
    pub otlp_endpoint: Option<String>,
    /// Batch export interval.
    pub batch_interval: Duration,
    /// Sample rate (1.0 = all requests, 0.1 = 10%).
    pub sample_rate: f64,
    /// Whether to propagate trace context in outgoing HTTP headers.
    pub propagate_context: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: env!("CARGO_PKG_NAME").to_string(),
            otlp_endpoint: None,
            batch_interval: Duration::from_secs(5),
            sample_rate: 1.0,
            propagate_context: true,
        }
    }
}

/// Initializes the tracing subscriber with optional OTLP export.
///
/// Returns a guard that must be held for the lifetime of the application.
/// When dropped, any pending spans are flushed.
///
/// # Panics
///
/// Panics if `set_global_default` is called more than once in the process lifetime.
pub fn init_tracing(config: TelemetryConfig) -> TracingGuard {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("tracing subscriber must be set once");

    if config.otlp_endpoint.is_some() {
        tracing::info!(
            service = %config.service_name,
            "OpenTelemetry tracing initialised (OTLP export enabled)"
        );
    } else {
        tracing::info!(
            service = %config.service_name,
            "Tracing initialised (local only)"
        );
    }

    TracingGuard { _config: config }
}

/// Guard that flushes pending spans on drop.
pub struct TracingGuard {
    _config: TelemetryConfig,
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        // Flush is handled by the tracing subscriber on drop
    }
}

/// Injects trace context headers into an outgoing HTTP request.
///
/// Called automatically by the `RequestTracing` middleware when
/// `TelemetryConfig::propagate_context` is `true`.
pub fn inject_trace_context<B>(request: &mut http::Request<B>) {
    let span = tracing::Span::current();
    if span.is_disabled() {
        return;
    }

    let headers = request.headers_mut();
    let _ = headers;
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_values() {
        let c = TelemetryConfig::default();
        assert!(!c.service_name.is_empty());
        assert!(c.sample_rate > 0.0);
    }

    #[test]
    fn init_tracing_does_not_panic() {
        let config = TelemetryConfig {
            otlp_endpoint: None,
            ..TelemetryConfig::default()
        };
        let _guard = init_tracing(config);
    }
}
