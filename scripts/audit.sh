#!/usr/bin/env bash
# Run dependency security checks locally.
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "→ Running cargo audit..."
if cargo audit 2>&1; then
    echo -e "  ${GREEN}✓${NC} cargo audit passed"
else
    echo -e "  ${RED}✗${NC} cargo audit found issues"
fi

echo ""
echo "→ Running cargo deny..."
if cargo deny check 2>&1; then
    echo -e "  ${GREEN}✓${NC} cargo deny passed"
else
    echo -e "  ${RED}✗${NC} cargo deny found issues"
fi

echo ""
echo "→ Running fuzz target (if cargo-fuzz is installed)..."
if command -v cargo-fuzz &> /dev/null; then
    if cargo fuzz run http_parse -- -max_total_time=30 2>&1; then
        echo -e "  ${GREEN}✓${NC} fuzz target passed"
    else
        echo -e "  ${RED}✗${NC} fuzz target found crashes"
    fi
else
    echo -e "  ${GREEN}(skipped, install cargo-fuzz with: cargo install cargo-fuzz)${NC}"
fi

echo ""
echo -e "${GREEN}Done.${NC}"
