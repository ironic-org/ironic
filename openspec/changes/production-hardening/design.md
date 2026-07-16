## Context

Ironic v0.4.8 has no CI/CD pipeline, no automated vulnerability scanning, no fuzz testing, and no standard operational endpoints. Every release requires manual `make release` runs. Production deployments must build their own version endpoint and health probes from scratch. The `HealthIndicator` trait conflates liveness (process alive) with readiness (dependencies healthy), making Kubernetes integration awkward.

## Goals / Non-Goals

**Goals:**
- Automated CI pipeline running on every PR and push to main: fmt, clippy, test (all-features), docs build, cargo audit, cargo deny
- Automated release workflow triggered by tag push: build, test, create GitHub Release
- Fuzz testing harness with at least one target covering HTTP request parsing
- `GET /version` endpoint returning build info (git SHA, timestamp, Rust version, active features)
- `GET /health/live` (liveness — always 200 if process is running) and `GET /health/ready` (readiness — checks dependencies)
- `HealthIndicator` trait split: `check_liveness()` and `check_readiness()` with default implementations
- `cargo audit` and `cargo deny` configurations checked in repo, runnable locally via script

**Non-Goals:**
- Not adding property-based testing (separate change)
- Not adding load-testing infrastructure (separate change)
- Not implementing secrets management vault integration
- Not adding SSRF protection middleware
- Not changing the existing `/health` endpoint behavior (backward compatible)

## Decisions

### Decision: GitHub Actions over GitLab CI / CircleCI
GitHub Actions is co-located with the GitHub repository, requires no additional service tokens, and the community is most familiar with it. The `.github/workflows/` directory is standard. If a user forks to GitLab, they can adapt the workflows.

### Decision: Separate liveness and readiness probe paths
Kubernetes convention expects `/healthz` (liveness) and `/readyz` (readiness). We expose `/health/live` and `/health/ready` to stay consistent with our existing `/health` prefix. The existing `/health` acts as a combined readiness check (backward compatible).

### Decision: `HealthIndicator` trait gains `check_liveness()` and `check_readiness()`
Rather than a breaking rename of `check()`, we add two methods with default implementations:
- `check_liveness()` — returns `Ok(())` by default (process is alive)
- `check_readiness()` — defaults to calling the old `check()`
Old `check()` is deprecated but not removed in this change.

### Decision: Build info injected at compile time via `env!("VERGEN_*")` or manual `build.rs`
Using `vergen` adds a dependency. Instead, we use a simple `build.rs` that reads `GIT_SHA`, `BUILD_TIMESTAMP` from env vars (set by CI) or falls back to `built::util::get_git_sha()` via the `built` crate. This is lighter weight and avoids pulling in an entire version-info ecosystem.

Alternative considered: `vergen` crate — rejected because it pulls in 15+ transitive deps for what is essentially env var injection.

### Decision: `cargo-fuzz` target uses `libfuzzer-sys` with HTTP fixture data
The fuzz target reads raw bytes and attempts to parse them as HTTP requests through the ironic-http extraction pipeline. Fixtures live in `fuzz/corpus/`. No network fuzzing (no DNS, no TLS) — purely stateless parsing fuzzing.

### Decision: `deny.toml` starts with conservative policies
- Allow list for licenses (MIT, Apache-2.0, BSD-3-Clause, ISC, Unicode-DFS-2016, Zlib)
- `deny` for GPL-3.0 and AGPL-3.0 (copyleft)
- Warnings for duplicates and multiple versions of the same crate
- Skip (warn) for workspace-level unmaintained advisories (some existing deps may have advisories)

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| `cargo audit` may flag existing deps with known advisories | Start with `warn` not `deny`; track in a `.advisory-deny` file |
| `cargo deny` may flag license violations in transitive deps | Pin known-safe versions; allow-list permissive licenses |
| Fuzz test may find real bugs immediately | Good — that's the point. Fix bugs as they surface, don't gate the change |
| `check_liveness()` / `check_readiness()` split may confuse existing users | Keep `check()` as deprecated alias for one release cycle; document migration |
| Build info env vars not available locally (no CI) | `build.rs` falls back to `"unknown"` for missing vars; users only see it in CI |
