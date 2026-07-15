---
title: "Request IDs and Tracing Spans — how observability bootstraps itself"
description: "How the RequestTracing middleware generates cryptographically unique request IDs, wires them into structured tracing spans, and propagates correlation context across every nested async operation in the Ironic framework."
date: "2026-07-15"
author: "Ironic Team"
---

# Request IDs and Tracing Spans — how observability bootstraps itself

Every HTTP server eventually hits the same debugging wall: a flood of interleaved log lines with zero correlation. Which log event belongs to which request? Did the database query on line 47 happen during the checkout call or the health check? Without an identity binding every log entry back to its originating request, you're lost.

Ironic's `RequestTracing` middleware answers this by running **first** in the request pipeline and doing three things atomically before your handler ever fires. Here's how it works, end to end.

## The middleware that runs before everything else

`RequestTracing` (`crates/ironic-http/src/observability.rs:47`) is a zero-sized struct implementing `Middleware`. When the framework compiles your application's request pipeline, `RequestTracing` is always injected as the outermost layer — it wraps every other middleware and your route handlers. This ordering is non-negotiable: tracing context must exist before any downstream code emits a log line.

The middleware's `handle` method is the critical path. On every incoming request it does three things, all before yielding to the next layer:

1. **Greedily reads or generates a request ID.** It checks for an incoming `x-request-id` header. If present and non-empty, it adopts the client-supplied value (enabling end-to-end correlation in distributed systems). If absent, it generates a fresh one.

2. **Inserts the `RequestId` into the request context.** The ID is stored as a typed extension on `RequestContext`, making it accessible to any handler or downstream middleware via `context.extension::<RequestId>()`.

3. **Creates a `tracing::info_span!`** with structured fields — `request_id`, `method`, and `uri` — that follow the OpenTelemetry HTTP semantic conventions.

All three happen synchronously in the same stack frame. There's no async suspension between reading the header and creating the span.

## The request ID format: `rf-{timestamp:032x}-{sequence:016x}`

Each generated ID is a 48-character hex string following the pattern `rf-{timestamp:032x}-{sequence:016x}`. It's built from two primitives:

```rust
fn generate() -> Self {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Self(format!("rf-{timestamp:032x}-{sequence:016x}"))
}
```

The timestamp is the number of nanoseconds since the Unix epoch, formatted as a zero-padded 32-hex-digit number. The sequence is a global `AtomicU64` counter starting at 1, formatted as 16 hex digits. The `rf-` prefix makes these IDs instantly recognizable in log aggregators as framework-generated request identifiers.

Why this design instead of a UUID? Three reasons:

- **No allocation for randomness.** UUID v4 requires CSPRNG bytes; this requires only a system call for the clock and an atomic increment. Both are cheaper than a UUID generation.
- **Human-readable prefix.** The leading `rf-` means you can grep for `rf-` and get every framework request ID without false positives.
- **Collision-free across restarts.** The 128-bit timestamp dominates the 64-bit sequence. Even if two processes start at the exact same nanosecond (vanishingly unlikely), the sequence counters differentiate them. After a restart, the wall clock advances, guaranteeing fresh IDs.

The full ID looks like: `rf-0000000196b3c7a0059a1b00-0000000000000001`. Long, but deliberately so — 48 hex characters encode more than enough entropy to be globally unique without external infrastructure.

## How the span propagates through async code

Once the span is created, the middleware box-pins an async future that calls `next.run(context)` and then attaches the response header. The key line:

```rust
.instrument(span)
```

This is `tracing::Instrument::instrument()`. It enters the span for the duration of the future, meaning **every** `tracing::info!()`, `debug!()`, `error!()` call inside any handler, any downstream middleware, or any awaited database query will be parented to this span — automatically, without any manual context passing. The span's `request_id` field appears as structured metadata on every child event.

When the response comes back, the middleware inserts the same `x-request-id` header into the response. The client gets the correlation token it sent (or the one the server generated), enabling the frontend to tie a specific HTTP response back to a reported error. This round-trip is critical for production debugging: support teams can ask a user for the `x-request-id` from their browser's Network panel and immediately locate the exact span in the tracing backend.

## OTLP export via the `telemetry` feature

Local spans are useful; distributed traces are transformative. Ironic gates OpenTelemetry Protocol (OTLP) export behind the `telemetry` feature flag, which activates `tracing-subscriber` with `env-filter` support.

Enable it and call `init_tracing` before building your application:

```rust
let _guard = init_tracing(TelemetryConfig {
    service_name: "my-api".into(),
    otlp_endpoint: Some("http://localhost:4317".into()),
    ..TelemetryConfig::default()
});
```

This wires the `tracing` subscriber as the global default with OTLP gRPC export. Every `RequestTracing` span — along with its method, URI, request ID, and all child events — streams to your collector (Jaeger, Tempo, Datadog). The `sample_rate` field (defaulting to `1.0`) controls what fraction of requests get exported, so you can dial it down in high-throughput environments without losing the structured-logging data.

`TracingGuard` is returned as a drop guard; when the application shuts down, pending spans get flushed. The request ID remains the unifying key across every span in the trace, bridging the gap between a log line in your terminal and a flame graph in your tracing UI.

The beauty of this design is that observability bootstraps itself. You write a handler with a `tracing::info!("processing order")` macro call, and Ironic automatically nests it under a span tagged with the request ID, HTTP method, and URI — with zero configuration beyond the initial `init_tracing` call. Distributed tracing stops being a bolt-on and becomes ambient infrastructure.
