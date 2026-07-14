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
#   1. Bumps version in Cargo.toml (workspace + internal deps)
#   2. Syncs all hardcoded version strings in docs/
#   3. Builds + runs full test suite + clippy
#   4. Creates git commit + tag
#   5. Pushes to GitHub (triggering CI)
#   6. Publishes to crates.io
# ──────────────────────────────────────────────────────────────────────

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_TOML="$ROOT/Cargo.toml"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

# ── helpers ──────────────────────────────────────────────────────────

workspace_version() {
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

# Replace <old> with <new> in a file (exact string, not regex)
sync_file() {
    local old="$1" new="$2" file="$3"
    if [[ ! -f "$file" ]]; then return; fi
    if grep -qF "$old" "$file" 2>/dev/null; then
        if [[ "$(uname)" == "Darwin" ]]; then
            sed -i '' "s/$old/$new/g" "$file"
        else
            sed -i "s/$old/$new/g" "$file"
        fi
        echo -e "  ${GREEN}✓${NC} $file"
    fi
}

# ── step 1: determine version ────────────────────────────────────────

CURRENT=$(workspace_version)
BUMP="${1:-}"

if [[ -n "$BUMP" ]]; then
    NEW=$(bump_version "$CURRENT" "$BUMP")
    echo -e "→ Bumping ${CYAN}v$CURRENT → v$NEW${NC} ($BUMP)"
else
    NEW="$CURRENT"
    echo -e "→ Releasing ${CYAN}v$NEW${NC}"
fi

# ── step 2: bump Cargo.toml if needed ────────────────────────────────

if [[ "$CURRENT" != "$NEW" ]]; then
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/version = \"$CURRENT\"/version = \"$NEW\"/" "$CARGO_TOML"
    else
        sed -i "s/version = \"$CURRENT\"/version = \"$NEW\"/" "$CARGO_TOML"
    fi
    echo -e "  ${GREEN}✓${NC} $CARGO_TOML"
fi

# ── step 3: sync internal deps to workspace version ──────────────────

CURRENT_DEP=$(grep 'ironic = { path = "."' "$CARGO_TOML" | sed 's/.*version = "\(.*\)".*/\1/')
if [[ -n "$CURRENT_DEP" ]] && [[ "$CURRENT_DEP" != "$NEW" ]]; then
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/ironic = { path = \".\", version = \"$CURRENT_DEP\"/ironic = { path = \".\", version = \"$NEW\"/" "$CARGO_TOML"
        sed -i '' "s/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$CURRENT_DEP\"/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$NEW\"/" "$CARGO_TOML"
    else
        sed -i "s/ironic = { path = \".\", version = \"$CURRENT_DEP\"/ironic = { path = \".\", version = \"$NEW\"/" "$CARGO_TOML"
        sed -i "s/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$CURRENT_DEP\"/ironic-macros = { path = \"crates\/ironic-macros\", version = \"$NEW\"/" "$CARGO_TOML"
    fi
    echo -e "  ${GREEN}✓${NC} internal deps synced ($CURRENT_DEP → $NEW)"
fi

# ── step 4: sync all docs to workspace version ───────────────────────

echo "→ Syncing docs to v$NEW"

DOC_FILES=(
    "$ROOT/docs/src/pages/home/components/hero-section.tsx"
    "$ROOT/docs/src/pages/home/components/stats-bar.tsx"
    "$ROOT/docs/content/docs/getting-started/getting-started.md"
    "$ROOT/docs/content/docs/getting-started/cli.md"
)

for f in "${DOC_FILES[@]}"; do
    # extract the first version-like string from the file
    DOC_VER=$(grep -oE '[0-9]+\.[0-9]+\.[0-9]+' "$f" | head -1)
    if [[ -n "$DOC_VER" ]] && [[ "$DOC_VER" != "$NEW" ]]; then
        sync_file "$DOC_VER" "$NEW" "$f"
    fi
done

# ── step 5: pre-flight checks ───────────────────────────────────────

echo "→ Running pre-flight checks..."

echo "  • cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "  • cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "  • cargo test --all-features"
cargo test --all-features

echo "  • npm run build (docs)"
npm --prefix "$ROOT/docs" run build

# ── step 6: git tag & push ───────────────────────────────────────────

echo "→ Creating git tag v$NEW"

cd "$ROOT"

git add Cargo.toml Cargo.lock \
    docs/src/pages/home/components/hero-section.tsx \
    docs/src/pages/home/components/stats-bar.tsx \
    docs/content/docs/getting-started/getting-started.md \
    docs/content/docs/getting-started/cli.md 2>/dev/null || true

if ! git diff --cached --quiet; then
    git commit -m "chore: release v$NEW"
    echo -e "  ${GREEN}✓${NC} committed"
else
    echo "  - nothing to commit"
fi

if git rev-parse "v$NEW" >/dev/null 2>&1; then
    git tag -d "v$NEW" 2>/dev/null || true
fi
git tag -a "v$NEW" -m "Release v$NEW"
echo -e "  ${GREEN}✓${NC} tag v$NEW created"

echo "→ Pushing to GitHub..."
git push origin HEAD
git push origin "v$NEW"

echo -e "  ${GREEN}✓${NC} pushed to origin"

# ── step 7: publish to crates.io ─────────────────────────────────────

echo "→ Publishing to crates.io..."

cargo publish -p ironic-macros --allow-dirty 2>&1 || echo "  ! ironic-macros publish skipped"
cargo publish -p ironic --allow-dirty

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║${NC}  🚀 Released ${CYAN}v$NEW${NC}"
echo -e "${GREEN}║${NC}"
echo -e "${GREEN}║${NC}  https://crates.io/crates/ironic/$NEW"
echo -e "${GREEN}║${NC}  https://github.com/ironic-org/ironic/releases/tag/v$NEW"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════╝${NC}"
