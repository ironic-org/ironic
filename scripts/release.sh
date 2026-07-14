#!/usr/bin/env bash
set -euo pipefail

# ── Ironic Release Script ───────────────────────────────────────────
# Usage:
#   ./scripts/release.sh              → release the current version
#   ./scripts/release.sh patch        → bump patch (0.1.8 → 0.1.9)
#   ./scripts/release.sh minor        → bump minor (0.1.8 → 0.2.0)
#   ./scripts/release.sh major        → bump major (0.1.8 → 1.0.0)
#
# Automatically:
#   1. Bumps version in Cargo.toml
#   2. Updates all hardcoded version strings in docs/
#   3. Builds + runs full test suite + clippy
#   4. Creates git commit + tag
#   5. Pushes to GitHub (triggering CI)
#   6. Publishes to crates.io
# ──────────────────────────────────────────────────────────────────────

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_TOML="$ROOT/Cargo.toml"

# ── helpers ──────────────────────────────────────────────────────────

current_version() {
    grep '^version = ' "$CARGO_TOML" | head -1 | sed 's/.*"\(.*\)".*/\1/'
}

bump_version() {
    local current="$1" bump="$2" major minor patch
    IFS='.' read -r major minor patch <<< "$current"
    case "$bump" in
        major) echo "$((major + 1)).0.0" ;;
        minor) echo "${major}.$((minor + 1)).0" ;;
        patch) echo "${major}.${minor}.$((patch + 1))" ;;
        *)     echo "unknown bump: $bump" >&2; exit 1 ;;
    esac
}

replace_version() {
    local old="$1" new="$2" file="$3"
    if grep -q "$old" "$file" 2>/dev/null; then
        # use sed -i '' for macOS, sed -i for Linux
        if [[ "$(uname)" == "Darwin" ]]; then
            sed -i '' "s/$old/$new/g" "$file"
        else
            sed -i "s/$old/$new/g" "$file"
        fi
        echo "  ✓ $file"
    fi
}

# ── step 1: determine version ────────────────────────────────────────

OLD_VERSION=$(current_version)
BUMP="${1:-}"

if [[ -n "$BUMP" ]]; then
    NEW_VERSION=$(bump_version "$OLD_VERSION" "$BUMP")
else
    NEW_VERSION="$OLD_VERSION"
fi

if [[ "$OLD_VERSION" == "$NEW_VERSION" ]]; then
    echo "→ Releasing v$NEW_VERSION (no bump)"
else
    echo "→ Bumping v$OLD_VERSION → v$NEW_VERSION ($BUMP)"

    # Update Cargo.toml workspace version
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
    else
        sed -i "s/version = \"$OLD_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
    fi

    # Also bump internal workspace dependency versions to match
    local old_dep_version
    old_dep_version=$(grep 'ironic = { path = "."' "$CARGO_TOML" | sed 's/.*version = "\(.*\)".*/\1/')
    if [[ -n "$old_dep_version" ]] && [[ "$old_dep_version" != "$NEW_VERSION" ]]; then
        if [[ "$(uname)" == "Darwin" ]]; then
            sed -i '' "s/ironic = { path = \".\", version = \"$old_dep_version\"/ironic = { path = \".\", version = \"$NEW_VERSION\"/" "$CARGO_TOML"
            sed -i '' "s/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$old_dep_version\"/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$NEW_VERSION\"/" "$CARGO_TOML"
        else
            sed -i "s/ironic = { path = \".\", version = \"$old_dep_version\"/ironic = { path = \".\", version = \"$NEW_VERSION\"/" "$CARGO_TOML"
            sed -i "s/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$old_dep_version\"/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$NEW_VERSION\"/" "$CARGO_TOML"
        fi
    fi
    echo "  ✓ $CARGO_TOML"
fi

# ── step 2: update hardcoded versions in docs ────────────────────────

echo "→ Updating docs from v$OLD_VERSION → v$NEW_VERSION"

replace_version "$OLD_VERSION" "$NEW_VERSION" \
    "$ROOT/docs/src/pages/home/components/hero-section.tsx"
replace_version "$OLD_VERSION" "$NEW_VERSION" \
    "$ROOT/docs/src/pages/home/components/stats-bar.tsx"
replace_version "$OLD_VERSION" "$NEW_VERSION" \
    "$ROOT/docs/content/docs/getting-started/getting-started.md"
replace_version "$OLD_VERSION" "$NEW_VERSION" \
    "$ROOT/docs/content/docs/getting-started/cli.md"

# ── step 3: pre-flight checks ────────────────────────────────────────

echo "→ Running pre-flight checks..."

echo "  • cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "  • cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "  • cargo test --all-features"
cargo test --all-features

echo "  • npm run build (docs)"
npm --prefix "$ROOT/docs" run build

# ── step 4: git tag & push ───────────────────────────────────────────

echo "→ Creating git tag v$NEW_VERSION"

cd "$ROOT"

if [[ "$OLD_VERSION" != "$NEW_VERSION" ]]; then
    git add Cargo.toml Cargo.lock \
        docs/src/pages/home/components/hero-section.tsx \
        docs/src/pages/home/components/stats-bar.tsx \
        docs/content/docs/getting-started/getting-started.md \
        docs/content/docs/getting-started/cli.md \
        docs/content/docs/observability.md 2>/dev/null || true

    git commit -m "chore: bump version to v$NEW_VERSION"
fi

if git rev-parse "v$NEW_VERSION" >/dev/null 2>&1; then
    echo "  ! tag v$NEW_VERSION already exists, skipping"
else
    git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"
    echo "  ✓ tag v$NEW_VERSION created"
fi

echo "→ Pushing to GitHub..."
git push origin HEAD
git push origin "v$NEW_VERSION"

echo "  ✓ pushed to origin"

# ── step 5: publish to crates.io ─────────────────────────────────────

echo "→ Publishing to crates.io..."

cargo publish -p ironic-macros --allow-dirty 2>&1 || echo "  ! ironic-macros publish skipped (may already be published)"
cargo publish -p ironic --allow-dirty

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  🚀 Released v$NEW_VERSION                                       ║"
echo "║                                                                  ║"
echo "║  https://crates.io/crates/ironic/$NEW_VERSION                     "
echo "║  https://github.com/ironic-org/ironic/releases/tag/v$NEW_VERSION  "
echo "╚══════════════════════════════════════════════════════════════════╝"
