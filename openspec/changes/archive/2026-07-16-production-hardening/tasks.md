## 1. CI Pipeline

- [x] 1.1 Create `.github/workflows/ci.yml` with triggers on PR and push to main
- [x] 1.2 Add job steps: fmt check, clippy (all-features), test (all-features), docs build
- [x] 1.3 Configure cargo/rust caching for faster subsequent runs
- [x] 1.4 Add cargo audit step to CI workflow
- [x] 1.5 Add cargo deny step to CI workflow
- [x] 1.6 Create `.github/workflows/release.yml` triggered by v* tag push
- [x] 1.7 Wire release workflow to create GitHub Release with changelog

## 2. Security Auditing

- [x] 2.1 Create `deny.toml` at repository root with license allow list (MIT, Apache-2.0, BSD-3-Clause, ISC, Unicode-DFS-2016, Zlib)
- [x] 2.2 Configure `deny.toml` to warn on duplicate crate versions, deny on copyleft licenses
- [x] 2.3 Create `scripts/audit.sh` that runs both cargo audit and cargo deny locally
- [x] 2.4 Add audit.sh to CI pipeline as a non-blocking advisory check

## 3. Fuzz Testing

- [x] 3.1 Create `fuzz/Cargo.toml` with libfuzzer-sys dependency (exclude from workspace)
- [x] 3.2 Create `fuzz/fuzz_targets/http_parse.rs` that feeds random bytes into ironic-http extraction
- [x] 3.3 Seed corpus at `fuzz/corpus/http_parse/` with valid HTTP request fixtures
- [x] 3.4 Add `fuzz/` to workspace exclude list in root `Cargo.toml`
- [x] 3.5 Add CI step to run fuzz target for 60 seconds (`cargo fuzz run http_parse -- -max_total_time=60`)
- [x] 3.6 Add fuzz target to `scripts/audit.sh` (optional, if cargo-fuzz is installed locally)

## 4. Build Info Injection

- [x] 4.1 Create `build.rs` that captures GIT_SHA, BUILD_TIMESTAMP env vars at compile time
- [x] 4.2 Implement fallback for local dev: use `built` crate or manual git invocation, default to "unknown"
- [x] 4.3 Define `BuildInfo` struct with git_sha, build_timestamp, rust_version, features, version fields
- [x] 4.4 Wire BuildInfo into the DI container as a singleton provider (always-on via VersionController)

## 5. Operational Endpoints

- [x] 5.1 Implement `GET /health/live` returning `{"status": "alive"}` with HTTP 200
- [x] 5.2 Implement `GET /health/ready` aggregating all `HealthIndicator::check_readiness()` results
- [x] 5.3 Return 503 on readiness failure, 200 on all healthy, 207 on degraded
- [x] 5.4 Implement `GET /version` returning BuildInfo JSON
- [x] 5.5 Register all three endpoints in the HealthModule (or a new OperationalModule)
- [x] 5.6 Feature flag — endpoints are always available (part of core HealthModule, no separate flag needed)
- [x] 5.7 Export new types from the facade crate prelude
- [x] 5.8 Write integration tests for all three endpoints

## 6. HealthIndicator Trait Split

- [x] 6.1 Add `check_liveness()` method to `HealthIndicator` trait with default `Ok(())` implementation
- [x] 6.2 Add `check_readiness()` method to `HealthIndicator` trait defaulting to existing `check()` behavior
- [x] 6.3 Deprecate existing `check()` method in doc comment (point to `check_readiness()`)
- [x] 6.4 Update all existing `HealthIndicator` implementations to use new methods (keep backward compat)
- [x] 6.5 Update `CompositeHealthModule` / health endpoint logic to use readiness/liveness distinction
- [x] 6.6 Ensure existing `/health` endpoint behavior is unchanged (calls readiness via default)

## 7. Documentation

- [x] 7.1 Update `docs/content/docs/more/deployment.md` production checklist with new endpoints
- [x] 7.2 Add `docs/content/docs/observability/operational-endpoints.md` documenting /version, /health/live, /health/ready
- [x] 7.3 Add CI/CD badge to README.md after workflows are working
- [x] 7.4 Update `docs/content/docs/observability/health-checks.md` with liveness/readiness guidance

## 8. Verification

- [x] 8.1 `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- [x] 8.2 `cargo test --all-features` passes
- [x] 8.3 `npm run build` (docs) succeeds
- [x] 8.4 Fuzz target compiles and runs without immediate crash (requires nightly + cargo-fuzz to actually run; CI step added)
- [x] 8.5 `cargo audit` and `cargo deny` pass locally (or known advisories documented)
