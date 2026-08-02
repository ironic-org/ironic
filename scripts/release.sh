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
if grep -q "^## \[v$NEW\] - " "$CHANGELOG" 2>/dev/null; then
    echo -e "  ${CYAN}!${NC} v$NEW entry already exists — skipping changelog insert"
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

# ── step 5: update releases pages from CHANGELOG.md ─────────────────

echo "→ Updating releases pages from CHANGELOG.md"

RELEASES_INDEX="$ROOT/docs/content/docs/releases/index.md"
CHANGELOG="$ROOT/CHANGELOG.md"
# Derive the major.minor series directory (e.g. v0.4.x from 0.4.1)
MAJOR_MINOR=$(echo "$NEW" | sed -E 's/^([0-9]+\.[0-9]+)\..*/\1/')
RELEASES_SERIES_DIR="$ROOT/docs/content/docs/releases/v${MAJOR_MINOR}.x"
RELEASES_V="$RELEASES_SERIES_DIR/index.md"

# Bump the "Current version:" line
sed -i '' "s/^## Current version: v[0-9.]*$/## Current version: v$NEW/" "$RELEASES_INDEX" 2>/dev/null || true

python3 -c "
import re, os

CHANGELOG = os.path.expanduser('$CHANGELOG')
RELEASES_INDEX = os.path.expanduser('$RELEASES_INDEX')
RELEASES_V = os.path.expanduser('$RELEASES_V')
MAJOR_MINOR = '$MAJOR_MINOR'

def parse_changelog(path):
    \"\"\"Parse CHANGELOG.md into (version_tuple, version, date, body) rows.\"\"\"
    with open(path) as f:
        text = f.read()
    # Split into sections: '## [vX.Y.Z] - YYYY-MM-DD' ... next '## ['
    sections = re.findall(
        r'^## \[v(\d+)\.(\d+)\.(\d+)\] - ([\d-]+)(.*?)(?=^## \[v|\\Z)',
        text,
        re.MULTILINE | re.DOTALL,
    )
    rows = []
    for mj, mn, pt, date, body in sections:
        ver = f'{mj}.{mn}.{pt}'
        body = body.strip()
        rows.append(((int(mj), int(mn), int(pt)), ver, date, body))
    return rows

def highlights(body):
    \"\"\"First bullet line of a changelog body.\"\"\"
    for line in body.splitlines():
        if line.startswith('- '):
            return line[2:].strip()
    return ''

rows = parse_changelog(CHANGELOG)

# ── Regenerate releases/index.md table ──────────────────────────────
with open(RELEASES_INDEX) as f:
    old = f.read()

if rows:
    rows.sort(key=lambda r: r[0], reverse=True)
    # Only list versions whose series page exists (v0.2.x has no page — skip)
    releases_dir = os.path.dirname(RELEASES_INDEX)
    table_rows = [
        f'| [v{slug}](/docs/releases/v{mj}.{mn}.x) | {date} | {highlights(body)} |'
        for (mj, mn, pt), slug, date, body in rows
        if os.path.isdir(os.path.join(releases_dir, f'v{mj}.{mn}.x'))
    ]
    marker_start = '| Version | Date | Highlights |\\n|---------|------|-----------|'
    marker_end = 'Full changelog:'
    idx_start = old.find(marker_start)
    idx_end = old.find(marker_end, idx_start)
    if idx_start >= 0 and idx_end >= 0:
        new_content = (
            old[:idx_start]
            + marker_start + '\\n'
            + '\\n'.join(table_rows) + '\\n\\n'
            + old[idx_end:]
        )
        with open(RELEASES_INDEX, 'w') as f:
            f.write(new_content)
        print('  \u2713 releases/index.md table regenerated from CHANGELOG.md')
    else:
        print('  ! markers not found in releases/index.md')
else:
    print('  ! no versioned sections found in CHANGELOG.md')

# ── Regenerate releases/vMAJOR.MINOR.x/index.md sections ──────────
if RELEASES_V and os.path.exists(RELEASES_V) and MAJOR_MINOR:
    with open(RELEASES_V) as f:
        series_content = f.read()
    series_rows = [
        r for r in rows
        if str(r[0][0]) + '.' + str(r[0][1]) == MAJOR_MINOR
    ]
    if series_rows:
        # Keep the intro (everything before the first version heading)
        intro_match = re.search(r'^## v', series_content, re.MULTILINE)
        if intro_match:
            intro = series_content[:intro_match.start()]
        else:
            intro_end = series_content.rfind('\\n---\\n')
            if intro_end > 0:
                intro = series_content[:intro_end + 5] + '\\n'
            else:
                intro = series_content + '\\n'
        # Build version sections from changelog bodies
        sections = []
        for ver_tuple, ver, date, body in sorted(series_rows, key=lambda r: r[0], reverse=True):
            if body:
                sections.append(f'## v{ver} \\u2014 {date}\\n\\n{body}\\n\\n---')
        if sections:
            new_series = intro + '\\n'.join(sections) + '\\n'
            with open(RELEASES_V, 'w') as f:
                f.write(new_series)
            print(f'  \u2713 {os.path.basename(RELEASES_V)} sections regenerated from CHANGELOG.md')
    else:
        print(f'  - no changelog sections for series v{MAJOR_MINOR}.x')
elif RELEASES_V and MAJOR_MINOR:
    print('  ! series file does not exist yet (will be created below)')
"

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
        echo "All versions in the v${MAJOR_MINOR}.x series."
        echo ""
        echo "---"
        echo ""
    } > "$RELEASES_V"
    echo -e "  ${GREEN}✓${NC} created $RELEASES_V with new series"
fi

# (series version sections are regenerated from CHANGELOG.md by the Python block above)

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

# ── step 7: commit & push (no tag) ──────────────────────────────────
# The tag is created and pushed by the CI release workflow
# (triggered manually via workflow_dispatch) only after verification
# and publish succeed.

echo "→ Committing and pushing v$NEW (tag will be created by CI)..."

cd "$ROOT"

git add -A

if ! git diff --cached --quiet; then
    git commit -m "chore: release v$NEW"
    echo -e "  ${GREEN}✓${NC} committed"
else
    echo "  - nothing to commit"
fi

echo "→ Pushing to current branch..."
if ! git push origin HEAD; then
    echo -e "  ${RED}✗${NC} failed to push to origin — aborting"
    exit 1
fi

echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║${NC}  🚀 Prepared ${CYAN}v$NEW${NC} for release"
echo -e "${GREEN}║${NC}  Commit pushed to main."
echo -e "${GREEN}║${NC}  CI will auto-detect the version bump and trigger the release workflow."
echo -e "${GREEN}║${NC}  Tag and crates.io publish will happen automatically after CI passes."
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════╝${NC}"
