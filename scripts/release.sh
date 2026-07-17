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
#   3. Creates blog post + updates releases pages
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

# ── step 4: sync current-version references in docs ───────────────────

echo "→ Syncing docs to v$NEW"

# Only update files that display the CURRENT version (not historical release notes/blogs)
while IFS= read -r -d '' f; do
    OLD_VER=$(grep -oE '[0-9]+\.[0-9]+\.[0-9]+' "$f" | head -1 || true)
    if [[ -n "$OLD_VER" ]] && [[ "$OLD_VER" != "$NEW" ]] && [[ "$OLD_VER" == "$CURRENT" ]]; then
        sync_file "$OLD_VER" "$NEW" "$f"
    fi
done < <(
    grep -rlE '[0-9]+\.[0-9]+\.[0-9]+' "$ROOT/docs" \
        --include='*.md' --include='*.tsx' --include='*.mdx' \
        --exclude-dir='blog' --exclude-dir='releases' 2>/dev/null || true
    # Also pick up the home page components (tsx not always in docs/)
    for f in "$ROOT/docs/src/pages/home/components/hero-section.tsx" \
             "$ROOT/docs/src/pages/home/components/stats-bar.tsx"; do
        if [[ -f "$f" ]]; then echo "$f"; fi
    done
) | sort -u | tr '\n' '\0'

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

# Generate a summary from the first meaningful changelog entry
FIRST_ENTRY=$( (echo -e "${added}${fixed}${changed}") | grep -oE '^- .+' | head -1 | sed 's/^- //' | sed 's/ ([a-f0-9]\{7\})$//' || true)
if [[ -z "$FIRST_ENTRY" ]]; then
    SUMMARY="Release v$NEW"
else
    SUMMARY="${FIRST_ENTRY:0:120}"
fi

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

# Update BlogIndex.tsx — bump "Latest: vX.Y.Z" badge
if grep -q "Latest: v[0-9]" "$BLOG_INDEX" 2>/dev/null; then
    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/Latest: v[0-9.]*/Latest: v$NEW/" "$BLOG_INDEX"
    else
        sed -i "s/Latest: v[0-9.]*/Latest: v$NEW/" "$BLOG_INDEX"
    fi
    echo -e "  ${GREEN}✓${NC} BlogIndex.tsx latest badge updated"
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

# Update releases/index.md — regenerate the full version table from all blog posts
echo "→ Regenerating releases version table from blog posts..."

# Bump the "Current version:" line
sed -i '' "s/^## Current version: v[0-9.]*$/## Current version: v$NEW/" "$RELEASES_INDEX" 2>/dev/null || true

python3 -c "
import re, os, glob

BLOG_DIR = os.path.expanduser('$BLOG_DIR')
RELEASES_INDEX = os.path.expanduser('$RELEASES_INDEX')
RELEASES_V = os.path.expanduser('$RELEASES_V')
MAJOR_MINOR = '$MAJOR_MINOR'

def parse_frontmatter(path):
    with open(path) as f:
        lines = f.read().splitlines()
    if not lines or lines[0] != '---':
        return {}
    end = 1
    while end < len(lines) and lines[end] != '---':
        end += 1
    front = {}
    for line in lines[1:end]:
        m = re.match(r'^(\w+):\s*\"(.+)\"$', line)
        if m:
            front[m.group(1)] = m.group(2)
    return front

def blog_body(path):
    \"\"\"Return everything after the frontmatter and title line.\"\"\"
    with open(path) as f:
        lines = f.read().splitlines()
    if not lines or lines[0] != '---':
        return ''
    end = 1
    while end < len(lines) and lines[end] != '---':
        end += 1
    body = lines[end+1:]
    # Skip the '# vX.Y.Z' title line
    while body and body[0].startswith('# '):
        body = body[1:]
    return '\n'.join(body).strip()

def format_date(d):
    try:
        parts = d.split('-')
        dt = calendar.timegm((int(parts[0]), int(parts[1]), int(parts[2], 0, 0, 0, 0)))
        from datetime import datetime
        dt = datetime.strptime(d, '%Y-%m-%d')
        return dt.strftime('%B %d, %Y')
    except Exception:
        return d

# Collect versioned blog posts
rows = []
for f in sorted(glob.glob(os.path.join(BLOG_DIR, 'v*.md'))):
    slug = os.path.splitext(os.path.basename(f))[0]
    m = re.match(r'^v(\d+)\.(\d+)\.(\d+)$', slug)
    if not m:
        continue
    front = parse_frontmatter(f)
    desc = front.get('description', slug)
    date = front.get('date', '')
    ver = f'{m.group(1)}.{m.group(2)}.{m.group(3)}'
    rows.append(((int(m.group(1)), int(m.group(2)), int(m.group(3))), ver, date, desc, f))

# ── Regenerate releases/index.md table ──────────────────────────────
with open(RELEASES_INDEX) as f:
    old = f.read()

# Preserve special entries (non-versioned or versioned without own blog post)
special = []
for line in old.splitlines():
    if line.startswith('| ['):
        slug_m = re.match(r'\| \[v(.+?)\]', line)
        if slug_m:
            vslug = slug_m.group(1)
            if not re.match(r'^\d+\.\d+\.\d+$', vslug):
                special.append(line)
            else:
                blog_path = os.path.join(BLOG_DIR, f'v{vslug}.md')
                if not os.path.exists(blog_path):
                    special.append(line)

seen = set()
unique_special = []
for line in special:
    if line not in seen:
        seen.add(line)
        unique_special.append(line)

if rows:
    rows.sort(key=lambda r: r[0], reverse=True)
    blog_rows = [
        f'| [v{slug}](/blog/v{slug}) | {date} | {desc} |'
        for _, slug, date, desc, _ in rows
    ]
    all_rows = blog_rows + unique_special
    marker_start = '| Version | Date | Highlights |\\n|---------|------|-----------|'
    marker_end = 'Full changelog:'
    idx_start = old.find(marker_start)
    idx_end = old.find(marker_end, idx_start)
    if idx_start >= 0 and idx_end >= 0:
        new_content = (
            old[:idx_start]
            + marker_start + '\\n'
            + '\\n'.join(all_rows) + '\\n\\n'
            + old[idx_end:]
        )
        with open(RELEASES_INDEX, 'w') as f:
            f.write(new_content)
        print('  \u2713 releases/index.md table regenerated')
    else:
        print('  ! markers not found in releases/index.md')
else:
    print('  ! no versioned blog posts found for releases table')

# ── Regenerate releases/vMAJOR.MINOR.x/index.md sections ──────────
if RELEASES_V and os.path.exists(RELEASES_V) and MAJOR_MINOR:
    with open(RELEASES_V) as f:
        series_content = f.read()
    # Find blog posts matching this major.minor series
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
            # Fallback: use everything after frontmatter as intro
            intro_end = series_content.rfind('\\n---\\n')
            if intro_end > 0:
                intro = series_content[:intro_end + 5] + '\\n'
            else:
                intro = series_content + '\\n'
        # Build version sections from blog posts
        sections = []
        for ver_tuple, ver, date, desc, fpath in sorted(series_rows, key=lambda r: r[0], reverse=True):
            body = blog_body(fpath)
            if body:
                human_date = format_date(date)
                sections.append(f'## v{ver} \\u2014 {human_date}\\n\\n{body}\\n\\n---')
        if sections:
            new_series = intro + '\\n'.join(sections) + '\\n'
            with open(RELEASES_V, 'w') as f:
                f.write(new_series)
            print(f'  \u2713 {os.path.basename(RELEASES_V)} sections regenerated')
    else:
        print(f'  - no blog posts for series v{MAJOR_MINOR}.x')
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
        echo "All versions in the v${MAJOR_MINOR}.x series. Visit the [Blog](/blog) for detailed release announcements."
        echo ""
        echo "---"
        echo ""
    } > "$RELEASES_V"
    echo -e "  ${GREEN}✓${NC} created $RELEASES_V with new series"
fi

# (series version sections are regenerated from blog posts by the Python block above)

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

# ── step 7: commit, tag & push ──────────────────────────────────────
# (crates.io publish is handled by GitHub Actions when the tag is pushed)

echo "→ Committing, tagging and pushing v$NEW..."

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

echo "→ Creating and pushing tag v$NEW..."
git tag -d "v$NEW" 2>/dev/null || true
git tag -a "v$NEW" -m "Release v$NEW"
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
echo -e "${GREEN}║${NC}  https://github.com/ironic-org/ironic/releases/tag/v$NEW"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════╝${NC}"
