---
name: release
description: Automate the Ironic release process. Bump version, update CHANGELOG, tag, and push. Use when the user wants to make a new release.
allowed-tools: Bash(git:*,cargo:*), Read, Write, Edit
license: MIT
metadata:
  author: ironic
  version: "1.0"
---

# Release

Automates the Ironic release workflow using Semantic Versioning and Keep a Changelog.

**Input**: Optionally specify a version bump (`major`, `minor`, `patch`, or a specific version like `0.2.0`). If omitted, prompt for the bump type.

**Steps**

1. **Determine current version**

   Read the current version from `Cargo.toml` under `[workspace.package]`:

   ```bash
   grep '^version' Cargo.toml | head -1
   ```

2. **Determine new version**

   If the user provided a bump type (`major`, `minor`, `patch`), compute the new version by parsing the current semver and incrementing the appropriate segment. If the user provided a specific version (e.g., `0.2.0`), use it directly. Otherwise prompt with options.

3. **Validate CHANGELOG**

   Read `CHANGELOG.md` and check:
   - The `## [Unreleased]` section exists and has content
   - If empty, warn the user before proceeding

4. **Update files**

   - Update version in `Cargo.toml`: `[workspace.package] version = "<new>"`
   - Update `CHANGELOG.md`: Replace `## [Unreleased]` with:
     ```
     ## [Unreleased]

     ## [<new-version>] - YYYY-MM-DD
     ```
   - Run `cargo check` to verify the workspace compiles

5. **Create release commit and tag**

   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: release v<new-version>"
   git tag v<new-version>
   ```

6. **Push (ask first)**

   Ask the user before pushing. If confirmed:

   ```bash
   git push origin main --tags
   ```

   The `.github/workflows/release.yml` will then run CI, create the GitHub Release, and publish to crates.io.

**Output On Success**

```
## Release v<new-version>

- Version bumped: <old> -> <new>
- CHANGELOG updated with release date
- Tag v<new-version> created
- Commit pushed: origin/main

CI will now: verify -> create GitHub Release -> publish to crates.io
```
