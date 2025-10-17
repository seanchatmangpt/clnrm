#!/usr/bin/env bash
# Verify clnrm init command behavior matches README claims
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR=$(mktemp -d)
CLNRM_BIN="${CLNRM_BIN:-/Users/sac/clnrm/target/release/clnrm}"

echo "🧪 Testing: clnrm init command"
echo "Test directory: $TEST_DIR"

cd "$TEST_DIR"

# Run clnrm init
echo "▶️ Running: clnrm init"
OUTPUT=$("$CLNRM_BIN" init 2>&1)

echo "📤 Output:"
echo "$OUTPUT"

# Verify output contains expected messages
echo ""
echo "✅ Verification checks:"

if echo "$OUTPUT" | grep -q "Initializing cleanroom test project"; then
    echo "  ✅ Contains initialization message"
else
    echo "  ❌ Missing initialization message"
    exit 1
fi

if echo "$OUTPUT" | grep -q "Project initialized successfully"; then
    echo "  ✅ Contains success message"
else
    echo "  ❌ Missing success message"
    exit 1
fi

# Verify file structure
echo ""
echo "📁 File structure verification:"

if [[ -f "tests/basic.clnrm.toml" ]]; then
    echo "  ✅ tests/basic.clnrm.toml created"
else
    echo "  ❌ tests/basic.clnrm.toml NOT created"
    exit 1
fi

if [[ -f "README.md" ]]; then
    echo "  ✅ README.md created"
else
    echo "  ❌ README.md NOT created"
    exit 1
fi

if [[ -d "scenarios" ]]; then
    echo "  ✅ scenarios/ directory created"
    SCENARIO_COUNT=$(find scenarios -type f | wc -l | tr -d ' ')
    echo "     ℹ️  scenarios/ contains $SCENARIO_COUNT file(s)"
else
    echo "  ❌ scenarios/ directory NOT created"
    exit 1
fi

# Verify TOML content
echo ""
echo "📋 TOML content verification:"

if grep -q '\[test\.metadata\]' tests/basic.clnrm.toml; then
    echo "  ✅ Contains [test.metadata] section"
else
    echo "  ❌ Missing [test.metadata] section"
    exit 1
fi

if grep -q 'name = "basic_test"' tests/basic.clnrm.toml; then
    echo "  ✅ Contains test name"
else
    echo "  ❌ Missing test name"
    exit 1
fi

if grep -q '\[\[steps\]\]' tests/basic.clnrm.toml; then
    echo "  ✅ Contains steps section"
else
    echo "  ❌ Missing steps section"
    exit 1
fi

# Verify TOML validates
echo ""
echo "🔍 TOML validation:"
VALIDATE_OUTPUT=$("$CLNRM_BIN" validate tests/ 2>&1)
if echo "$VALIDATE_OUTPUT" | grep -qE "(valid|Configuration valid)"; then
    echo "  ✅ TOML validates successfully"
else
    echo "  ❌ TOML validation failed"
    echo "  Output: $VALIDATE_OUTPUT"
    exit 1
fi

# Cleanup
cd /
rm -rf "$TEST_DIR"

echo ""
echo "✅ All README claims about 'clnrm init' verified successfully!"
