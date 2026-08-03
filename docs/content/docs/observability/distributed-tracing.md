---
title: Distributed Tracing
description: Propagate trace context across service boundaries
---

# Distributed Tracing

Ironic automatically propagates W3C trace context across microservice
boundaries. When the `telemetry` feature is enabled, it uses OpenTelemetry's
`traceparent`/`tracestate` headers. Without telemetry, it falls back to
basic `tracing` span propagation.

## How It Works

Each transport envelope includes trace headers:

```
Envelope
├── correlation_id
├── route
├── headers
│   ├── traceparent: 00-{trace_id}-{span_id}-01
│   └── tracestate: ...
└── payload
```

The `inject_trace_context()` function adds the current span context to
outgoing messages. The `extract_trace_context()` function on the receiving
side creates a child span linked to the incoming trace.

## HTTP Client

The `HttpClientService` automatically injects a `traceparent` header into
all outbound HTTP requests.

## Graph

```
Service A          Service B          Service C
  │                    │                    │
  ├── request ────────▶│                    │
  │  (traceparent)     ├── request ────────▶│
  │                    │  (traceparent)     │
  │                    │◀─── response ──────│
  │◀─── response ──────│                    │
  │                    │                    │
```

All spans share the same `trace_id`, creating a single trace across
service boundaries.
