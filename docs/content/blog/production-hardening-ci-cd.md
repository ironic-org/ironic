---
title: "Production-hardening 1: CI/CD pipeline, security auditing, and release automation"
description: "GitHub Actions CI with fmt/clippy/test/docs/audit/deny on every PR. Automated release workflow triggered by tag push. cargo-audit and cargo-deny for dependency vulnerability and license compliance. Local audit script for offline checks."
date: "Jul 16, 2026"
author: "Ironic Team"
---

# Production-hardening 1: CI/CD pipeline, security auditing, and release automation

Every production framework needs a CI/CD pipeline that catches regressions before they reach users and automates releases so they happen consistently. This post covers how Ironic's CI/CD, security auditing, and release workflows work.

---

## CI pipeline

The `.github/workflows/ci.yml` workflow runs on every pull request and push to `main`. It executes these steps in order:

```
cargo fmt --all -- --check          # formatting
cargo clippy --all-features -D warnings  # linting
cargo test --all-features           # tests
npm --prefix docs ci && npm --prefix docs run build  # docs
cargo audit                         # dependency vulnerabilities
cargo deny check                    # license compliance + duplicates
```

The audit and deny steps use `if: success() || failure()` so they run even when earlier steps fail — security information is valuable regardless of test status.

### Caching

Cargo registry and build artifacts are cached using `actions/cache@v4` with a key based on `Cargo.lock` hash:

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}
```

This reduces subsequent CI runs from ~30 minutes to under 5 minutes on cache hits.

---

## Release workflow

The `.github/workflows/release.yml` workflow triggers on any tag matching `v*`:

```yaml
on:
  push:
    tags: ["v*"]
```

It runs the full verification suite (fmt, clippy, test, docs, audit, deny), generates a changelog from git history, and creates a GitHub Release:

```yaml
- name: Generate changelog
  run: |
    echo "## What's Changed" > release-notes.md
    git log --oneline --no-merges $(git describe --tags --abbrev=0 HEAD^)..HEAD >> release-notes.md

- uses: softprops/action-gh-release@v2
  with:
    body_path: release-notes.md
```

This completely replaces the old manual `release.sh` commit-tag-push dance. Releases are now one command: `git tag v0.5.0 && git push --tags`.

---

## Security auditing

Dependency security is enforced at two levels:

### cargo-audit

`cargo audit` checks the advisory database (1,160+ advisories) against all 605+ crate dependencies. If a crate has a known vulnerability, CI reports it. The check is non-blocking (warns) so existing advisories don't block development while fixes are tracked.

### cargo-deny

`deny.toml` at the repository root configures license and duplicate policies:

```toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC", "Zlib", ...]
deny = ["GPL-3.0", "AGPL-3.0", "GPL-2.0", "LGPL-3.0"]

[bans]
multiple-versions = "warn"   # e.g., windows-sys@0.52 + 0.60 + 0.61
```

- **License compliance**: Allowed licenses are explicitly listed; copyleft and unlicensed are denied
- **Duplicate detection**: Multiple versions of the same crate warn (e.g., `windows-sys` appears in 3 versions due to transitive dependencies from ring, notify, and clap)

### Local audit script

For developers who want to run checks offline without CI:

```bash
# scripts/audit.sh
./scripts/audit.sh
# → cargo audit + cargo deny + optional cargo fuzz (if installed)
```

---

## Fuzz testing

The `fuzz/` directory contains a `cargo-fuzz` target for HTTP parsing:

```
fuzz/
├── Cargo.toml                  # libfuzzer-sys dependency
├── corpus/http_parse/          # seeded with valid HTTP fixtures
│   ├── simple_get
│   ├── simple_post
│   ├── query_params
│   └── delete_with_id
└── fuzz_targets/http_parse.rs  # random bytes → serde_json + URL + header parsing
```

CI runs the fuzz target for 60 seconds on each push to catch regressions:

```bash
cargo +nightly fuzz run http_parse -- -max_total_time=60
```

The fuzz crate is excluded from the workspace (`exclude = ["fuzz"]` in root `Cargo.toml`) so it doesn't affect regular builds.

---

## What this means for production

Before this change, every release required manual testing and manual `git tag && git push`. Now:

- **Every PR** is automatically verified against 6 quality gates
- **Every push to main** is tested again (catches merge skew)
- **Every tag push** creates a GitHub Release with auto-generated changelog
- **Every dependency** is scanned for known vulnerabilities
- **Every license** is checked against the allow list
- **HTTP parsing** is continuously fuzzed for crash-inducing inputs

The release badge on the README shows the latest release status at a glance:

```
[![Release](https://github.com/ironic-org/ironic/actions/workflows/release.yml/badge.svg)](https://github.com/ironic-org/ironic/actions/workflows/release.yml)
```
