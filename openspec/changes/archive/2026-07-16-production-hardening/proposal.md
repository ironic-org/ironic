## Why

Ironic has zero CI/CD pipeline, no automated security vulnerability scanning, no fuzz testing, and no standard operational endpoints (version, readiness, liveness). Every release is built and tested manually. Every production deployment must cobble together its own operational tooling. This blocks the framework from being truly production-ready.

## What Changes

- **GitHub Actions CI pipeline** — run clippy, test (all-features), docs build, cargo audit, cargo deny on every PR and push to main
- **cargo audit integration** — automated dependency vulnerability scanning in CI; `cargo deny` for license compliance and duplicate crate detection
- **Fuzz testing harness** — `cargo-fuzz` target for HTTP request parsing (multipart, JSON, URL params)
- **Version/info endpoint** — `GET /version` returning git SHA, build timestamp, Rust version, active features
- **Readiness/liveness probes** — `GET /health/live` (process alive) and `GET /health/ready` (dependencies healthy) endpoints
- **cargo-audit and cargo-deny as dev-dependencies** — with `scripts/audit.sh` for local runs
- **Release workflow** — GitHub Actions release workflow triggered by tag push, replaces local `release.sh` for the commit+tag+push step

## Capabilities

### New Capabilities
- `ci-pipeline`: GitHub Actions workflows for PR checks (fmt, clippy, test, docs, audit, deny) and release publishing
- `security-auditing`: Automated dependency vulnerability scanning with cargo-audit and license/policy checks with cargo-deny
- `fuzz-testing`: cargo-fuzz harness targeting HTTP request parsing paths (multipart, JSON, URL params, headers)
- `operational-endpoints`: GET /version (build info), GET /health/live (liveness), GET /health/ready (readiness) endpoints

### Modified Capabilities
- `observability`: Add liveness/readiness probe semantics to the health check system; add build-info metadata to the existing health module
- `composite-health`: Extend HealthIndicator trait with a `readiness()` method alongside the existing `check()` for liveness

## Impact

- **New dependencies**: `cargo-audit`, `cargo-deny`, `cargo-fuzz` (CI only); `serde` for version endpoint serialization (already a dep)
- **New files**: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `deny.toml`, `fuzz/Cargo.toml`, `fuzz/fuzz_targets/http_parse.rs`, `scripts/audit.sh`
- **Modified files**: `crates/ironic-core/src/health.rs`, `crates/ironic-http/src/lib.rs`, `Cargo.toml` (features), `src/lib.rs` (facade exports)
- **New crate features**: `operational-endpoints` (or fold under existing `devtools`/`observability`)
- **BREAKING**: `HealthIndicator::check()` return type may change to distinguish liveness from readiness; existing implementations need a `readiness()` method
