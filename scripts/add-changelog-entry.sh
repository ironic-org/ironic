#!/usr/bin/env bash
set -euo pipefail

# ── Add Changelog Entry ─────────────────────────────────────────────
# Usage:
#   ./scripts/add-changelog-entry.sh "Added"    "Feature description"
#   ./scripts/add-changelog-entry.sh "Fixed"    "Bug fix description"
#   ./scripts/add-changelog-entry.sh "Changed"  "Refactoring description"
#   ./scripts/add-changelog-entry.sh "Security" "Security fix description"
#
# Categories (case-insensitive): Added, Fixed, Changed, Security
# If omitted, defaults to "Changed".
# ────────────────────────────────────────────────────────────────────

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CHANGELOG="$ROOT/CHANGELOG.md"

CATEGORY="${1:-Changed}"
ENTRY_TEXT="${2:-}"

if [[ -z "$ENTRY_TEXT" ]]; then
    echo "Usage: $0 [Category] \"Entry text\""
    echo "Categories: Added, Fixed, Changed, Security (default: Changed)"
    exit 1
fi

CATEGORY="$(tr '[:lower:]' '[:upper:]' <<< "${CATEGORY:0:1}")${CATEGORY:1}"

case "$CATEGORY" in
    Added|Fixed|Changed|Security) ;;
    *) echo "Invalid category: $CATEGORY (valid: Added, Fixed, Changed, Security)"; exit 1 ;;
esac

if ! grep -q "^## \[Unreleased\]" "$CHANGELOG" 2>/dev/null; then
    echo "Error: CHANGELOG.md is missing the [Unreleased] section header."
    exit 1
fi

# Locate Unreleased boundaries (absolute line numbers)
UNRELEASED_LINE=$(grep -n "^## \[Unreleased\]" "$CHANGELOG" | head -1 | cut -d: -f1)
NEXT_HEADER_LINE=$(tail -n +$((UNRELEASED_LINE + 1)) "$CHANGELOG" \
    | grep -n "^## \[" | head -1 | cut -d: -f1 || true)

if [[ -n "$NEXT_HEADER_LINE" ]]; then
    UNRELEASED_END=$((UNRELEASED_LINE + NEXT_HEADER_LINE))
else
    UNRELEASED_END=$(( $(wc -l < "$CHANGELOG") + 1 ))
fi

UNRELEASED_TAIL=$((UNRELEASED_END - 1))

# Check if category exists in Unreleased section
CAT_HEADER_REL=$(sed -n "${UNRELEASED_LINE},${UNRELEASED_END}p" "$CHANGELOG" \
    | grep -n "^### ${CATEGORY}$" | head -1 | cut -d: -f1 || true)

if [[ -n "$CAT_HEADER_REL" ]]; then
    CAT_HEADER_LINE=$((UNRELEASED_LINE + CAT_HEADER_REL - 1))
    # Find last non-blank entry under this category
    LAST_ENTRY=$(awk '
        /^## \[Unreleased\]/ {f=1; next}
        f && /^### / {cur=$2; next}
        f && /^## \[/ {exit}
        f && cur == "'"$CATEGORY"'" && NF {saved=NR}
        END {print saved ? saved : 0}
    ' "$CHANGELOG")
    if [[ "$LAST_ENTRY" -gt 0 ]]; then
        # Append after last entry
        sed -i '' "${LAST_ENTRY}a\\
- ${ENTRY_TEXT}" "$CHANGELOG"
    else
        # Category header exists but no entries — append after header
        sed -i '' "${CAT_HEADER_LINE}a\\
- ${ENTRY_TEXT}" "$CHANGELOG"
    fi
else
    # New category — insert before Unreleased section end
    sed -i '' "${UNRELEASED_TAIL}i\\
\\
### ${CATEGORY}\\
- ${ENTRY_TEXT}" "$CHANGELOG"
fi

echo "  ✓ Added to [Unreleased] > ${CATEGORY}: ${ENTRY_TEXT}"
