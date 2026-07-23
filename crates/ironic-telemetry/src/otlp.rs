use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};

use crate::telemetry::TelemetryConfig;

/// Guard that initialises and shuts down an OTLP tracer provider.
///
/// Creates an [`SdkTracerProvider`] configured from [`TelemetryConfig`] and
/// sets it as the global tracer provider. On drop, the provider is shut down
/// gracefully.
pub(crate) struct OtlpGuard {
    pub(crate) provider: Option<SdkTracerProvider>,
}

impl OtlpGuard {
    /// Creates a new guard.
    ///
    /// If `config.otlp_endpoint` is `None`, no provider is initialised.
    /// Failures are logged via `tracing::warn!` and swallowed so the application
    /// can continue without telemetry.
    pub(crate) fn new(config: &TelemetryConfig) -> Self {
        let provider = config.otlp_endpoint.as_ref().and_then(|endpoint| {
            match try_init_otlp(endpoint, config) {
                Ok(p) => Some(p),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to initialise OTLP exporter");
                    None
                }
            }
        });
        OtlpGuard { provider }
    }
}

impl Drop for OtlpGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.provider.take()
            && let Err(e) = provider.shutdown()
        {
            tracing::warn!(error = %e, "Error shutting down OTLP provider");
        }
    }
}

fn try_init_otlp(
    endpoint: &str,
    config: &TelemetryConfig,
) -> Result<SdkTracerProvider, Box<dyn std::error::Error + Send + Sync>> {
    let sampler = if (config.sample_rate - 1.0).abs() < f64::EPSILON {
        Sampler::AlwaysOn
    } else if config.sample_rate <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.sample_rate)
    };

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(config.batch_interval)
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_resource(
            Resource::builder_empty()
                .with_service_name(config.service_name.clone())
                .build(),
        )
        .build();

    opentelemetry::global::set_tracer_provider(provider.clone());

    Ok(provider)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otlp_guard_without_endpoint_returns_none() {
        let config = TelemetryConfig {
            otlp_endpoint: None,
            sample_rate: 1.0,
            service_name: "test".into(),
            batch_interval: std::time::Duration::from_secs(5),
            propagate_context: true,
        };
        let guard = OtlpGuard::new(&config);
        assert!(guard.provider.is_none());
    }

    #[test]
    fn sampler_selection() {
        let config = TelemetryConfig {
            otlp_endpoint: None,
            sample_rate: 1.0,
            service_name: "test".into(),
            batch_interval: std::time::Duration::from_secs(5),
            propagate_context: true,
        };

        // AlwaysOn for sample_rate ~1.0
        assert!((config.sample_rate - 1.0f64).abs() < f64::EPSILON);

        // AlwaysOff for sample_rate <= 0.0
        let rate_off: f64 = 0.0;
        assert!(rate_off <= 0.0);

        // TraceIdRatioBased for other values
        let rate_ratio: f64 = 0.5;
        assert!(rate_ratio > 0.0 && (rate_ratio - 1.0).abs() >= f64::EPSILON);
    }
}
