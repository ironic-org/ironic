use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};

use crate::telemetry::TelemetryConfig;

pub(crate) struct OtlpGuard {
    pub(crate) provider: Option<SdkTracerProvider>,
}

impl OtlpGuard {
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
