#![allow(clippy::type_complexity)]
//! Distributed tracing context propagation for microservice envelopes.
//!
//! Automatically injects and extracts W3C `traceparent`/`tracestate` headers
//! from transport envelopes when the `telemetry` feature is enabled. Without
//! `telemetry`, uses `tracing` span IDs for basic propagation.

use std::collections::BTreeMap;

use crate::distributed::microservices::Envelope;

#[allow(dead_code)]
const TRACEPARENT_HEADER: &str = "traceparent";
#[allow(dead_code)]
const TRACESTATE_HEADER: &str = "tracestate";

/// Injects the current tracing context into the envelope headers.
///
/// Call this before sending a message via a transport to propagate the
/// current trace across service boundaries.
pub fn inject_trace_context(envelope: &mut Envelope) {
    #[cfg(feature = "telemetry")]
    inject_w3c_otel(&mut envelope.headers);

    #[cfg(not(feature = "telemetry"))]
    inject_tracing_span(&mut envelope.headers);
}

/// Extracts tracing context from envelope headers and returns a guard that
/// should be held for the duration of message processing.
///
/// Call this at the start of a message handler to link the incoming trace
/// to the current processing context.
pub fn extract_trace_context(
    envelope: &Envelope,
) -> Option<PropagatedSpan> {
    #[cfg(feature = "telemetry")]
    return extract_w3c_otel(&envelope.headers);

    #[cfg(not(feature = "telemetry"))]
    extract_tracing_span(&envelope.headers)
}

/// A guard that holds a reference to a propagated tracing span.
/// Drop it when message processing completes.
pub struct PropagatedSpan {
    #[cfg(feature = "telemetry")]
    _otel_guard: Option<opentelemetry::Context>,
    #[cfg(not(feature = "telemetry"))]
    _tracing_span: Option<tracing::Span>,
}

// ---------------------------------------------------------------------------
// Basic tracing propagation (uses tracing span fields)
// ---------------------------------------------------------------------------

#[cfg(not(feature = "telemetry"))]
fn inject_tracing_span(headers: &mut BTreeMap<String, String>) {
    let span = tracing::Span::current();
    let id = span.id().map(|id| id.into_u64()).unwrap_or(0);
    headers.insert(TRACEPARENT_HEADER.to_string(), format!("00-{id:x}-{id:x}-01"));
}

#[cfg(not(feature = "telemetry"))]
fn extract_tracing_span(headers: &BTreeMap<String, String>) -> Option<PropagatedSpan> {
    if let Some(traceparent) = headers.get(TRACEPARENT_HEADER) {
        let span = tracing::info_span!("extracted", %traceparent);
        Some(PropagatedSpan {
            _tracing_span: Some(span),
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// W3C OpenTelemetry propagation (uses opentelemetry crate)
// ---------------------------------------------------------------------------

#[cfg(feature = "telemetry")]
fn inject_w3c_otel(headers: &mut BTreeMap<String, String>) {
    use opentelemetry::global;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let _cx = tracing::Span::current().context();
    global::get_text_map_propagator(|propagator| {
        propagator.inject(&mut HeaderInjector(headers))
    });
}

#[cfg(feature = "telemetry")]
fn extract_w3c_otel(headers: &BTreeMap<String, String>) -> Option<PropagatedSpan> {
    use opentelemetry::global;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let parent_cx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(headers))
    });
    let span = tracing::info_span!("extracted");
    let cx = parent_cx.clone();
    span.set_parent(cx);
    Some(PropagatedSpan {
        _otel_guard: Some(parent_cx),
    })
}

#[cfg(feature = "telemetry")]
struct HeaderInjector<'a>(&'a mut BTreeMap<String, String>);

#[cfg(feature = "telemetry")]
impl opentelemetry::propagation::Injector for HeaderInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }
}

#[cfg(feature = "telemetry")]
struct HeaderExtractor<'a>(&'a BTreeMap<String, String>);

#[cfg(feature = "telemetry")]
impl opentelemetry::propagation::Extractor for HeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|s| s.as_str())
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|s| s.as_str()).collect()
    }
}
