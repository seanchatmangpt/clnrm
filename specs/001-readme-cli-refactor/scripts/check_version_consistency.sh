#!/bin/bash
# Version Consistency Check Script
# Ensures README version matches Cargo.toml and uses badges only
#
# Exit codes:
# 0 = GREEN (version consistent)
# 1 = RED (version mismatch or hardcoded versions)

set -e

echo "🔍 Checking version consistency..."
echo ""

# Extract Cargo.toml workspace version
CARGO_VERSION=$(grep -A 2 '^\[workspace.package\]' Cargo.toml | grep '^version' | sed 's/version = "\(.*\)"/\1/')

echo "Cargo.toml version: $CARGO_VERSION"

# Check if version badge exists in README
if grep -q "img.shields.io/crates/v/clnrm" README.md; then
    echo "✅ Version badge present in README"
else
    echo "❌ Version badge missing in README"
    echo "   Expected: ![Version](https://img.shields.io/crates/v/clnrm.svg)"
    exit 1
fi

# Check for hardcoded version numbers (excluding badge URLs and GitHub links)
echo ""
echo "Checking for hardcoded version numbers..."

# Exclude badge URLs and GitHub links from search
if grep -v "img.shields.io" README.md | \
   grep -v "github.com/seanchatmangpt" | \
   grep -v "crates.io" | \
   grep -E "v?$CARGO_VERSION" > /dev/null 2>&1; then
    echo "❌ Hardcoded version '$CARGO_VERSION' found in README"
    echo "   Only version badges should contain version numbers"
    echo ""
    echo "Found in:"
    grep -v "img.shields.io" README.md | grep -v "github.com" | grep -E "v?$CARGO_VERSION" || true
    exit 1
else
    echo "✅ No hardcoded version numbers (badges only)"
fi

# Verify clnrm --version matches Cargo.toml (if binary exists)
if command -v clnrm &> /dev/null; then
    BINARY_VERSION=$(clnrm --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
    echo ""
    echo "Binary version: $BINARY_VERSION"

    if [ "$BINARY_VERSION" = "$CARGO_VERSION" ]; then
        echo "✅ Binary version matches Cargo.toml"
    else
        echo "⚠️  Binary version mismatch (expected $CARGO_VERSION, got $BINARY_VERSION)"
        echo "   Run 'cargo build' to rebuild with correct version"
    fi
else
    echo ""
    echo "ℹ️  clnrm binary not found (skipping binary version check)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Version consistency check PASSED"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Summary:"
echo "  - Cargo.toml version: $CARGO_VERSION"
echo "  - README uses version badge: ✅"
echo "  - No hardcoded versions: ✅"
