---
title: Error Counter Metric
description: Built-in ironic_http_errors_total counter — auto-incremented on every 5xx error, exported in Prometheus scrape.
---

# Error Counter Metric

## What is it?

`ironic_http_errors_total` is a Prometheus counter that auto-increments on every 5xx response. No configuration needed — it's built into the metrics layer.

## How to use

Enable the `metrics` feature and add the layer:

```rust
AxumAdapter::new()
    .configure_router(|r| r.layer(MetricsLayer::new(MetricsConfig::default())))
```

Then scrape:

```
# HELP ironic_http_errors_total Total HTTP errors (5xx)
# TYPE ironic_http_errors_total counter
ironic_http_errors_total 42
```

## Alerting

Create a Prometheus alert for error rate spikes:

```yaml
- alert: HighErrorRate
  expr: rate(ironic_http_errors_total[5m]) > 1
  for: 2m
  annotations:
    summary: "Error rate above 1/sec for 2 minutes"
```

## Per-endpoint status breakdown

`ironic_http_endpoint_status_total` tracks 2xx, 4xx, 5xx counts per endpoint:

```
ironic_http_endpoint_status_total{endpoint="GET /api/users",status="2xx"} 1500
ironic_http_endpoint_status_total{endpoint="GET /api/users",status="4xx"} 12
ironic_http_endpoint_status_total{endpoint="GET /api/users",status="5xx"} 3
```

Use this to pinpoint which endpoints are erroring:

```yaml
- alert: EndpointErrorRate
  expr: rate(ironic_http_endpoint_status_total{status="5xx"}[5m]) > 0.1
  annotations:
    summary: "Endpoint {{ $labels.endpoint }} has elevated 5xx rate"
```
