# Release Process

## Branch Strategy

```
feature/*  →  main  →  release  →  crates.io + tag
```

| Branch | Purpose | CI Trigger |
|--------|---------|------------|
| `main` | All PRs merge here | On PR only |
| `release` | Only receives `main` merges; triggers publish | On push |

## Daily Development

1. Create a PR from `feature/*` targeting `main`
2. CI runs: fmt, clippy, test (stable + nightly), audit, deny, fuzz
3. Merge PR to `main` — no CI re-run (already tested on PR)

## Making a Release

### Step 1 — Bump version on `main`

Update version in `Cargo.toml` via a PR:

```bash
git checkout -b bump-v1.0.3
# Edit Cargo.toml: version = "1.0.3"
git commit -m "chore: bump version to 1.0.3"
git push origin bump-v1.0.3
# Create PR → merge to main
```

Also update any secondary version references (e.g. `docs/lib/constants.ts` if applicable).

### Step 2 — Merge `main` → `release`

```bash
git checkout release
git pull origin release
git merge main
git push origin release
```

Pushing to `release` triggers **Release** automatically:

1. **check-version** — Reads `Cargo.toml`, compares with latest git tag, skips if unchanged
2. **verify** — `cargo audit` + `cargo deny`
3. **publish** — Publishes `ironic-macros` → `ironic` to crates.io, generates release notes, creates git tag, creates GitHub Release

## Workflow Files

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | CI on PR to `main` (fmt, clippy, test, audit, deny, fuzz, docs build) |
| `.github/workflows/release.yml` | Release on push to `release` or manual dispatch |
| `.github/workflows/docs.yml` | Docs build + deploy to GitHub Pages |

## Release Workflow (`release.yml`)

```yaml
on:
  push:
    branches: [release]
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., v1.0.1)'
        required: true
```

Jobs run sequentially: `check-version` → `verify` → `publish`.

- On `push` to `release`: version detected from `Cargo.toml`, compared against latest git tag. Skips if unchanged.
- On `workflow_dispatch` (manual): version taken from input, always runs.

## First-Time Setup

```bash
# Create the release branch (one time)
git checkout main && git pull
git checkout -b release
git push origin release
```

## Emergency Release

Go to **GitHub → Actions → Release → Run workflow** → enter version tag (e.g. `v1.0.3`).

## Branch Protection (Recommended)

### `main` branch
- ☑ Require a pull request before merging
- ☑ Require approvals (1)
- ☑ Require status checks (CI checks must pass)
- ☑ Do not allow bypass

### `release` branch
- ☑ Require a pull request before merging (only you merge `main → release`)
- ☑ Do not allow bypass
