#!/bin/bash
# scripts/install_hooks.sh
# Install pre-commit hooks for telemetry validation
# Usage: ./scripts/install_hooks.sh

set -euo pipefail

HOOK_SRC="scripts/pre-commit.sh"
HOOK_DEST=".git/hooks/pre-commit"

echo "📦 Installing Pre-Commit Hooks"
echo ""

# Check if in git repo
if [ ! -d ".git" ]; then
    echo "❌ Not in a git repository"
    exit 1
fi

# Check if hook source exists
if [ ! -f "$HOOK_SRC" ]; then
    echo "❌ Hook source not found: $HOOK_SRC"
    exit 1
fi

# Backup existing hook
if [ -f "$HOOK_DEST" ]; then
    BACKUP="${HOOK_DEST}.backup.$(date +%s)"
    echo "📦 Backing up existing hook to $BACKUP"
    cp "$HOOK_DEST" "$BACKUP"
fi

# Install hook
cp "$HOOK_SRC" "$HOOK_DEST"
chmod +x "$HOOK_DEST"

echo "✅ Pre-commit hook installed at $HOOK_DEST"
echo ""
echo "Usage:"
echo "  - Hook runs automatically on: git commit"
echo "  - Bypass with: git commit --no-verify"
echo "  - Uninstall: rm .git/hooks/pre-commit"
echo ""
echo "The hook will:"
echo "  ✓ Validate telemetry against registry schemas"
echo "  ✓ Run tests for changed files"
echo "  ✓ Block commits with violations"
echo ""
echo "Test it now: git commit --allow-empty -m 'test hook'"
