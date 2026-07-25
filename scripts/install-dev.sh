#!/usr/bin/env bash
set -euo pipefail

# ── Ironic Dev Install Script ──────────────────────────────────────────
# Installs the latest local development version of the Ironic CLI.
# Run this after pulling changes to use `ironic` CLI with the latest code.
#
# Usage:
#   ./scripts/install-dev.sh              # Build & install CLI
#   ./scripts/install-dev.sh --debug      # Build with debug symbols
#   ./scripts/install-dev.sh --force      # Force reinstall (skip cache)
# ────────────────────────────────────────────────────────────────────────

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROFILE="${1:---debug}"
FORCE=""

if [[ "$*" == *"--force"* ]]; then
    FORCE="--force"
fi

echo "→ Installing Ironic CLI from local source..."
echo "  Source: $ROOT"
echo "  Target: $(which ironic 2>/dev/null || echo "$HOME/.cargo/bin/ironic")"

cargo install --path . $PROFILE $FORCE

echo ""
echo "✓ Ironic CLI installed successfully!"
echo ""
echo "  Version: $(ironic --version 2>/dev/null || echo 'unknown')"
echo ""
echo "  Quick start:"
echo "    ironic new my-platform          # Create a monorepo workspace"
echo "    cd my-platform"
echo "    ironic generate app auth-service # Add a microservice"
echo "    ironic start -p api-gateway      # Run the API gateway"
