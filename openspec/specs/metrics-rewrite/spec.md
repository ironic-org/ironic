# Metrics Rewrite

## Purpose

Lock-free histogram metrics, per-endpoint labels, public Counter/Gauge/Histogram API for user code.

## Requirements

### Requirement: Metrics SHALL use lock-free histogram bucketing
The framework SHALL record latency into pre-configured histogram buckets at request completion time using atomic operations, not a Mutex-guarded Vec.

#### Scenario: Latency bucketed at record time
- **WHEN** a request completes
- **THEN** its latency SHALL be recorded into the appropriate histogram bucket via atomic increment

### Requirement: System SHALL provide public Counter, Gauge, and Histogram API
Application code SHALL be able to register and update custom metrics through a public API.

#### Scenario: Custom counter registered and incremented
- **WHEN** application code calls `MetricsRegistry::counter("my_metric")`
- **AND** increments it
- **THEN** the counter value SHALL appear in the `/metrics` scrape output

### Requirement: System SHALL support per-endpoint metrics labels
When `MetricsConfig.per_endpoint` is true, the `/metrics` output SHALL include `{method="GET",path="/users"}` labels.

#### Scenario: Per-endpoint labels emitted
- **WHEN** `per_endpoint` is `true`
- **AND** a `GET /users` request completes
- **THEN** the metrics output SHALL contain `ironic_http_requests_total{method="GET",path="/users"}`

### Requirement: Metrics store SHALL NOT grow unboundedly
The latency storage SHALL use a bounded ring buffer (configurable size, default 1000) for percentile computation instead of an unbounded Vec.

#### Scenario: Bounded ring buffer
- **WHEN** more than 1000 requests are recorded
- **THEN** the ring buffer SHALL overwrite the oldest entry instead of growing
