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
#   1. Checks if version already published on crates.io — aborts if so
#   2. Bumps version in Cargo.toml (workspace + internal deps)
#   3. Syncs all hardcoded version strings in docs/
#   4. Generates CHANGELOG.md from git commits since last tag
#   5. Creates blog post + updates releases pages
#   6. Builds + runs full test suite + clippy
#   7. Publishes to crates.io
#   8. Creates git commit + tag
#   9. Pushes to GitHub (triggering CI)
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

# Check if a version is already published on crates.io
is_version_published() {
    local ver="$1"
    local body
    body=$(curl -sf "https://crates.io/api/v1/crates/ironic" 2>/dev/null) || return 1
    echo "$body" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    max_ver = data.get('crate', {}).get('max_version', '')
    print(max_ver)
except Exception:
    print('')
" 2>/dev/null | grep -qF "$ver"
}

# ── step 0: check if version already published ───────────────────────

echo "→ Checking crates.io for current version..."

CURRENT=$(workspace_version)
BUMP="${1:-}"

if [[ -n "$BUMP" ]]; then
    NEW=$(bump_version "$CURRENT" "$BUMP")
    echo -e "→ Bumping ${CYAN}v$CURRENT → v$NEW${NC} ($BUMP)"
else
    NEW="$CURRENT"
    echo -e "→ Releasing ${CYAN}v$NEW${NC}"
fi

if is_version_published "$NEW"; then
    echo -e "  ${RED}✗${NC} v$NEW is already published on crates.io — aborting"
    echo "  Run with 'patch'/'minor'/'major' to bump first."
    exit 1
fi
echo -e "  ${GREEN}✓${NC} v$NEW is not yet published — proceeding"

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
        feat:*)     added="${added}$(format_entry "$msg" "$hash")\n" ;;
        feat\(*:*)  added="${added}$(format_entry "$msg" "$hash")\n" ;;
        fix:*)      fixed="${fixed}$(format_entry "$msg" "$hash")\n" ;;
        fix\(*:*)   fixed="${fixed}$(format_entry "$msg" "$hash")\n" ;;
        docs:*)     changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        docs\(*:*)  changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        chore:*)    changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        chore\(*:*) changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        refactor:*) changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        refactor\(*:*) changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        test:*)     changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        test\(*:*)  changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        perf:*)     changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        perf\(*:*)  changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
        security:*) security="${security}$(format_entry "$msg" "$hash")\n" ;;
        security\(*:*) security="${security}$(format_entry "$msg" "$hash")\n" ;;
        *)          changed="${changed}$(format_entry "$msg" "$hash")\n" ;;
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

# Check for duplicate entry before inserting
if grep -q "^## \[v$NEW\] - " "$CHANGELOG" 2>/dev/null; then
    echo -e "  ${CYAN}!${NC} v$NEW entry already exists — skipping changelog insert"
else
    # Insert after the [Unreleased] section header using temp file
    if grep -q "## \[Unreleased\]" "$CHANGELOG" 2>/dev/null; then
        head_line=$(grep -n "## \[Unreleased\]" "$CHANGELOG" | head -1 | cut -d: -f1)
        {
            head -n "$head_line" "$CHANGELOG"
            echo ""
            echo "$ENTRY"
            tail -n +$((head_line + 1)) "$CHANGELOG"
        } > "${CHANGELOG}.tmp"
        mv "${CHANGELOG}.tmp" "$CHANGELOG"
        echo -e "  ${GREEN}✓${NC} CHANGELOG.md updated"
    else
        echo "  ! CHANGELOG.md not found or missing [Unreleased] section"
    fi
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
    DOC_VER=$(grep -oE '[0-9]+\.[0-9]+\.[0-9]+' "$f" | head -1 || true)
    if [[ -n "$DOC_VER" ]] && [[ "$DOC_VER" != "$NEW" ]]; then
        sync_file "$DOC_VER" "$NEW" "$f"
    fi
done

# ── step 5: create blog post and update releases ────────────────────

echo "→ Creating blog post for v$NEW"

BLOG_DIR="$ROOT/docs/content/blog"
BLOG_FILE="$BLOG_DIR/v$NEW.md"
BLOG_INDEX="$ROOT/docs/src/pages/BlogIndex.tsx"
RELEASES_INDEX="$ROOT/docs/content/docs/releases/index.md"
# Derive the major.minor series directory (e.g. v0.4.x from 0.4.1)
MAJOR_MINOR=$(echo "$NEW" | sed -E 's/^([0-9]+\.[0-9]+)\..*/\1/')
RELEASES_SERIES_DIR="$ROOT/docs/content/docs/releases/v${MAJOR_MINOR}.x"
RELEASES_V="$RELEASES_SERIES_DIR/index.md"

# Format the changelog sections for the blog post
format_blog_section() {
    local title="$1" items="$2"
    if [[ -n "$items" ]]; then
        echo ""
        echo "### $title"
        echo "$items"
    fi
}

BLOG_BODY=""
BLOG_BODY="${BLOG_BODY}$(format_blog_section "Added" "$added")"
BLOG_BODY="${BLOG_BODY}$(format_blog_section "Fixed" "$fixed")"
BLOG_BODY="${BLOG_BODY}$(format_blog_section "Changed" "$changed")"
BLOG_BODY="${BLOG_BODY}$(format_blog_section "Security" "$security")"

# Generate a summary from the first change entry
FIRST_LINE=$(grep -oE '^- .+' <<< "$COMMITS" | head -1 | sed 's/^- //' | sed 's/ (.*//' || echo "Release v$NEW")
SUMMARY="${FIRST_LINE:0:120}"

# Create blog post only if it doesn't already exist
if [[ -f "$BLOG_FILE" ]]; then
    echo -e "  ${CYAN}!${NC} blog post already exists — skipping"
else
    cat > "$BLOG_FILE" << BLOGEOF
---
title: "v$NEW — $SUMMARY"
description: "$SUMMARY"
date: "$TODAY"
author: "Ironic Team"
---

# v$NEW
$BLOG_BODY
BLOGEOF

    echo -e "  ${GREEN}✓${NC} blog post created: $BLOG_FILE"
fi

# Update BlogIndex.tsx — insert new post after the opening array bracket, skip if exists
if grep -q "const posts: Post\[\] = \[" "$BLOG_INDEX" 2>/dev/null; then
    if grep -q "slug: 'v$NEW'" "$BLOG_INDEX" 2>/dev/null; then
        echo -e "  ${CYAN}!${NC} BlogIndex.tsx already has v$NEW — skipping"
    else
        NEW_POST_ENTRY="    {
        slug: 'v$NEW',
        title: 'v$NEW — $SUMMARY',
        description: '$SUMMARY',
        date: '$TODAY',
        tag: 'release',
        readTime: '2 min',
    },"
        POSTS_LINE=$(grep -n "const posts: Post\[\] = \[" "$BLOG_INDEX" | head -1 | cut -d: -f1 || true)
        if [[ -n "$POSTS_LINE" ]]; then
            {
                head -n "$POSTS_LINE" "$BLOG_INDEX"
                echo "$NEW_POST_ENTRY"
                tail -n +$((POSTS_LINE + 1)) "$BLOG_INDEX"
            } > "$BLOG_INDEX.tmp" && mv "$BLOG_INDEX.tmp" "$BLOG_INDEX"
            echo -e "  ${GREEN}✓${NC} BlogIndex.tsx updated"
        fi
    fi
else
    echo "  ! BlogIndex.tsx pattern not found — add manually"
fi

# Update releases/index.md — add row to the version table & bump current version, skip if exists
if grep -q "| \[v$NEW\]" "$RELEASES_INDEX" 2>/dev/null; then
    echo -e "  ${CYAN}!${NC} releases/index.md already has v$NEW — skipping"
else
    # Bump the "Current version:" line
    if grep -q "^## Current version: " "$RELEASES_INDEX" 2>/dev/null; then
        if [[ "$(uname)" == "Darwin" ]]; then
            sed -i '' "s/^## Current version: v[0-9.]*$/## Current version: v$NEW/" "$RELEASES_INDEX"
        else
            sed -i "s/^## Current version: v[0-9.]*$/## Current version: v$NEW/" "$RELEASES_INDEX"
        fi
    fi
    # Add row to version table (insert before first existing version row)
    TABLE_INSERT="| [v$NEW](/blog/v$NEW) | $TODAY | $SUMMARY |"
    RELEASES_TABLE_LINE=$(grep -n "| v" "$RELEASES_INDEX" | head -1 | cut -d: -f1 || true)
    if [[ -n "$RELEASES_TABLE_LINE" ]]; then
        {
            head -n "$((RELEASES_TABLE_LINE - 1))" "$RELEASES_INDEX"
            echo "$TABLE_INSERT"
            tail -n +"$RELEASES_TABLE_LINE" "$RELEASES_INDEX"
        } > "$RELEASES_INDEX.tmp" && mv "$RELEASES_INDEX.tmp" "$RELEASES_INDEX"
        echo -e "  ${GREEN}✓${NC} releases/index.md updated"
    fi
fi

# Format date for human-readable release section headers
format_date() {
    local d="$1"
    if [[ "$(uname)" == "Darwin" ]]; then
        date -j -f "%Y-%m-%d" "$d" "+%B %d, %Y" 2>/dev/null || echo "$d"
    else
        date -d "$d" "+%B %d, %Y" 2>/dev/null || echo "$d"
    fi
}

RELEASE_DATE=$(format_date "$TODAY")

# Create releases series directory if it doesn't exist (e.g. v0.4.x/)
# When a major/minor bump occurs, create the new series file from a template
if [[ ! -f "$RELEASES_V" ]]; then
    mkdir -p "$RELEASES_SERIES_DIR"
    # Find the previous series directory
    PREV_SERIES=$(find "$ROOT/docs/content/docs/releases" -maxdepth 1 -type d -name 'v*.x' \
        | sed 's/.*\/v\([0-9.]*\).x/\1/' | sort -t. -k1,1n -k2,2n | tail -1)
    # Mark the previous series as no longer current (e.g. "Current Stable Series" → "Stable Series")
    if [[ -n "$PREV_SERIES" ]]; then
        PREV_FILE="$ROOT/docs/content/docs/releases/v${PREV_SERIES}.x/index.md"
        if [[ -f "$PREV_FILE" ]]; then
            if [[ "$(uname)" == "Darwin" ]]; then
                sed -i '' 's/— Current Stable Series$/— Stable Series (Legacy)/' "$PREV_FILE"
                sed -i '' 's/stable series\.$/stable series (legacy)./' "$PREV_FILE"
            else
                sed -i 's/— Current Stable Series$/— Stable Series (Legacy)/' "$PREV_FILE"
                sed -i 's/stable series\.$/stable series (legacy)./' "$PREV_FILE"
            fi
            echo -e "  ${GREEN}✓${NC} v${PREV_SERIES}.x marked as legacy"
        fi
    fi
    {
        echo "---"
        echo "title: v${MAJOR_MINOR}.x"
        echo "description: Complete changelog and release notes for the Ironic v${MAJOR_MINOR}.x stable series."
        echo "---"
        echo ""
        echo "# v${MAJOR_MINOR}.x — Current Stable Series"
        echo ""
        echo "All versions in the v${MAJOR_MINOR}.x series. Visit the [Blog](/blog) for detailed release announcements."
        echo ""
        echo "---"
        echo ""
    } > "$RELEASES_V"
    echo -e "  ${GREEN}✓${NC} created $RELEASES_V with new series"
fi

# Find the first existing version entry to use as the insertion anchor
RELEASES_V_ANCHOR=$(grep -E "^## v" "$RELEASES_V" | head -1 | sed 's/^## //; s/ —.*//' || true)

# Update releases series index — prepend new version section, skip if exists
if grep -q "^## v$NEW — " "$RELEASES_V" 2>/dev/null; then
    echo -e "  ${CYAN}!${NC} $(basename "$RELEASES_V") already has v$NEW — skipping"
else
    RELEASES_V_INSERT="## v$NEW — $RELEASE_DATE
$BLOG_BODY

---

"
    if [[ -n "$RELEASES_V_ANCHOR" ]]; then
        ANCHOR_LINE=$(grep -n "^## v$RELEASES_V_ANCHOR" "$RELEASES_V" | head -1 | cut -d: -f1 || true)
    else
        # No version entries yet — anchor after the last --- (intro separator)
        ANCHOR_LINE=$(grep -n "^---$" "$RELEASES_V" | tail -1 | cut -d: -f1 || true)
        if [[ -n "$ANCHOR_LINE" ]]; then
            ANCHOR_LINE=$((ANCHOR_LINE + 1))
        else
            ANCHOR_LINE=1
        fi
    fi
    if [[ -n "$ANCHOR_LINE" ]]; then
        {
            head -n "$((ANCHOR_LINE - 1))" "$RELEASES_V"
            echo "$RELEASES_V_INSERT"
            tail -n +"$ANCHOR_LINE" "$RELEASES_V"
        } > "$RELEASES_V.tmp" && mv "$RELEASES_V.tmp" "$RELEASES_V"
        echo -e "  ${GREEN}✓${NC} releases/v${MAJOR_MINOR}.x/index.md updated"
    fi
fi

# ── step 6: pre-flight checks ───────────────────────────────────────

echo "→ Running pre-flight checks..."

echo "  • cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "  • cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "  • cargo test --all-features"
cargo test --all-features

echo "  • npm run build (docs)"
npm --prefix "$ROOT/docs" run build

# ── step 7: publish to crates.io ────────────────────────────────────

echo "→ Publishing to crates.io..."

cargo publish -p ironic-macros --allow-dirty 2>&1 || echo "  ! ironic-macros publish skipped"
cargo publish -p ironic --allow-dirty

# ── step 8: git tag & push ───────────────────────────────────────────

echo "→ Creating git tag v$NEW"

cd "$ROOT"

git add Cargo.toml Cargo.lock CHANGELOG.md \
    docs/src/pages/home/components/hero-section.tsx \
    docs/src/pages/home/components/stats-bar.tsx \
    docs/content/docs/getting-started/getting-started.md \
    docs/content/docs/getting-started/cli.md \
    docs/content/blog/v$NEW.md \
    docs/src/pages/BlogIndex.tsx \
    docs/content/docs/releases/index.md \
    "$RELEASES_V" 2>/dev/null || true

if ! git diff --cached --quiet; then
    git commit -m "chore: release v$NEW"
    echo -e "  ${GREEN}✓${NC} committed"
else
    echo "  - nothing to commit"
fi

# Always delete stale local tag before creating a new one
git tag -d "v$NEW" 2>/dev/null || true
git tag -a "v$NEW" -m "Release v$NEW"
echo -e "  ${GREEN}✓${NC} tag v$NEW created"

echo "→ Pushing to GitHub..."
if ! git push origin HEAD; then
    echo -e "  ${RED}✗${NC} failed to push to origin — aborting"
    git tag -d "v$NEW" 2>/dev/null || true
    exit 1
fi

echo "→ Pushing tag v$NEW..."
if git push origin "v$NEW" 2>&1; then
    echo -e "  ${GREEN}✓${NC} tag pushed"
else
    echo -e "  ${CYAN}!${NC} tag already exists on remote — force pushing..."
    git push --force origin "v$NEW"
    echo -e "  ${GREEN}✓${NC} tag force-pushed"
fi

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║${NC}  🚀 Released ${CYAN}v$NEW${NC}"
echo -e "${GREEN}║${NC}"
echo -e "${GREEN}║${NC}  https://crates.io/crates/ironic/$NEW"
echo -e "${GREEN}║${NC}  https://github.com/ironic-org/ironic/releases/tag/v$NEW"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════╝${NC}"
