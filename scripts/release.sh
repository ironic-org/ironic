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
#   2. Generates CHANGELOG.md from git commits since last tag
#   3. Updates the releases pages (docs/content/docs/releases/) from CHANGELOG.md
#   4. Runs pre-flight checks (fmt, clippy, all-features tests, docs build)
#   5. Commits, tags, and pushes to GitHub
#      (crates.io publish is handled by GitHub Actions on tag push)
# ──────────────────────────────────────────────────────────────────────

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CARGO_TOML="$ROOT/Cargo.toml"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
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

CURRENT=$(workspace_version)
BUMP="${1:-}"

if [[ -n "$BUMP" ]]; then
    NEW=$(bump_version "$CURRENT" "$BUMP")
    echo -e "→ Bumping ${CYAN}v$CURRENT → v$NEW${NC} ($BUMP)"
else
    NEW="$CURRENT"
    echo -e "→ Releasing ${CYAN}v$NEW${NC}"
fi

# ── step 1: bump Cargo.toml if needed ────────────────────────────────

if [[ "$CURRENT" != "$NEW" ]]; then
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/version = \"$CURRENT\"/version = \"$NEW\"/" "$CARGO_TOML"
    else
        sed -i "s/version = \"$CURRENT\"/version = \"$NEW\"/" "$CARGO_TOML"
    fi
    echo -e "  ${GREEN}✓${NC} $CARGO_TOML"
fi

# ── step 2: sync internal deps to workspace version ──────────────────

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

# ── step 3: generate changelog ────────────────────────────────────

echo "→ Generating changelog for v$NEW"

TODAY=$(date +%Y-%m-%d)
CHANGELOG="$ROOT/CHANGELOG.md"

# Extract [Unreleased] section content (everything between ## [Unreleased] and next ## [ header)
UNRELEASED_RAW=$(sed -n '/^## \[Unreleased\]/,/^## \[/p' "$CHANGELOG" 2>/dev/null || echo "")
UNRELEASED_BODY=$(echo "$UNRELEASED_RAW" | tail -n +2 | sed '$d' | sed '/^$/d')

if [[ -n "$(echo "$UNRELEASED_BODY" | tr -d '[:space:]')" ]]; then
    echo "  • Using [Unreleased] section content (skipping git log)"
    ENTRY="## [v${NEW}] - ${TODAY}
${UNRELEASED_BODY}"
    USING_UNRELEASED=true
else
    PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

    if [[ -n "$PREV_TAG" ]]; then
        COMMITS=$(git log --oneline --no-merges "${PREV_TAG}..HEAD" 2>/dev/null || echo "")
    else
        COMMITS=$(git log --oneline --no-merges 2>/dev/null || echo "")
    fi

    # Parse commits into categories. Strips conventional commit prefix for clean output.
    added=""
    fixed=""
    changed=""
    security=""

    strip_prefix() {
        sed -E 's/^[a-z]+(\([^)]*\))?:[[:space:]]*//' <<< "$1"
    }

    format_entry() {
        local msg="$1" hash="$2"
        local clean; clean=$(strip_prefix "$msg")
        echo "- ${clean} (${hash:0:7})"
    }

    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        msg=$(echo "$line" | sed 's/^[a-f0-9]* //')
        hash=$(echo "$line" | awk '{print $1}')

        case "$msg" in
            feat:*)     added="${added}$(format_entry "$msg" "$hash")"$'\n' ;;
            feat\(*:*)  added="${added}$(format_entry "$msg" "$hash")"$'\n' ;;
            fix:*)      fixed="${fixed}$(format_entry "$msg" "$hash")"$'\n' ;;
            fix\(*:*)   fixed="${fixed}$(format_entry "$msg" "$hash")"$'\n' ;;
            docs:*)     changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            docs\(*:*)  changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            chore:*)    changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            chore\(*:*) changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            refactor:*) changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            refactor\(*:*) changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            test:*)     changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            test\(*:*)  changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            perf:*)     changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            perf\(*:*)  changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
            security:*) security="${security}$(format_entry "$msg" "$hash")"$'\n' ;;
            security\(*:*) security="${security}$(format_entry "$msg" "$hash")"$'\n' ;;
            *)          changed="${changed}$(format_entry "$msg" "$hash")"$'\n' ;;
        esac
    done <<< "$COMMITS"

    # Build new changelog entry with real newlines
    ENTRY="## [v${NEW}] - ${TODAY}
"
    [[ -n "$added" ]] && ENTRY="${ENTRY}
### Added
${added}"
    [[ -n "$fixed" ]] && ENTRY="${ENTRY}
### Fixed
${fixed}"
    [[ -n "$changed" ]] && ENTRY="${ENTRY}
### Changed
${changed}"
    [[ -n "$security" ]] && ENTRY="${ENTRY}
### Security
${security}"

    if [[ -z "$added" && -z "$fixed" && -z "$changed" && -z "$security" ]]; then
        ENTRY="${ENTRY}
- Initial release
"
    fi
    USING_UNRELEASED=false
fi

# Check for duplicate entry before inserting
if grep -Eq "^## \[(v)?$NEW\]" "$CHANGELOG" 2>/dev/null; then
    echo -e "  ${CYAN}!${NC} v$NEW entry already exists — skipping changelog insert"
elif ! grep -q "^## \[Unreleased\]" "$CHANGELOG" 2>/dev/null; then
    echo "  ! CHANGELOG.md has no [Unreleased] section; add the next release entry manually"
else
    # Insert after the [Unreleased] section header using temp file
    if grep -q "## \[Unreleased\]" "$CHANGELOG" 2>/dev/null; then
        head_line=$(grep -n "## \[Unreleased\]" "$CHANGELOG" | head -1 | cut -d: -f1)
        if [[ "$USING_UNRELEASED" == "true" ]]; then
            # Skip stale Unreleased body — find the next version header
            next_line=$(tail -n +$((head_line + 1)) "$CHANGELOG" \
                | grep -n '^## \[' | head -1 | cut -d: -f1)
            if [[ -n "$next_line" ]]; then
                tail_start=$((head_line + next_line))
            else
                tail_start=$((head_line + 1))
            fi
            {
                head -n "$head_line" "$CHANGELOG"
                echo ""
                echo "$ENTRY"
                tail -n +"$tail_start" "$CHANGELOG"
            } > "${CHANGELOG}.tmp"
        else
            {
                head -n "$head_line" "$CHANGELOG"
                echo ""
                echo "$ENTRY"
                tail -n +$((head_line + 1)) "$CHANGELOG"
            } > "${CHANGELOG}.tmp"
        fi
        mv "${CHANGELOG}.tmp" "$CHANGELOG"
        echo -e "  ${GREEN}✓${NC} CHANGELOG.md updated"
    else
        echo "  ! CHANGELOG.md not found or missing [Unreleased] section"
    fi
fi

# ── step 4: sync current-version references in docs ───────────────────

echo "→ Syncing version constant to v$NEW"
if [[ -f "$ROOT/docs/lib/constants.ts" ]]; then
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/export const CURRENT_VERSION = 'v\?[0-9.]*';/export const CURRENT_VERSION = '$NEW';/" "$ROOT/docs/lib/constants.ts"
    else
        sed -i "s/export const CURRENT_VERSION = 'v\?[0-9.]*';/export const CURRENT_VERSION = '$NEW';/" "$ROOT/docs/lib/constants.ts"
    fi
    echo -e "  ${GREEN}✓${NC} $ROOT/docs/lib/constants.ts"
fi

# ── step 5: sync first-release documentation ────────────────────────

echo "→ Syncing first-release documentation"
FIRST_RELEASE="$ROOT/docs/content/docs/first-release.md"
if [[ -f "$FIRST_RELEASE" ]]; then
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' -E "s/(# Ironic )[0-9.]+/\\1$NEW/; s/(starts with Ironic )[0-9.]+/\\1$NEW/" "$FIRST_RELEASE"
    else
        sed -i -E "s/(# Ironic )[0-9.]+/\\1$NEW/; s/(starts with Ironic )[0-9.]+/\\1$NEW/" "$FIRST_RELEASE"
    fi
    echo -e "  ${GREEN}✓${NC} $FIRST_RELEASE"
fi

# ── step 6: pre-flight checks ───────────────────────────────────────

echo "→ Running pre-flight checks..."

echo "  • cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "  • cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "  • cargo test --all-features"
cargo test --all-features

echo "  • bun run build (docs)"
bun install --frozen-lockfile --cwd "$ROOT/docs" && bun run --cwd "$ROOT/docs" build

# ── step 7: hand off ────────────────────────────────────────────────

echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║${NC}  🚀 Prepared ${CYAN}v$NEW${NC} for release"
echo -e "${GREEN}║${NC}  Commit, tag, and push manually when ready."
echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"

