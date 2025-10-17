#!/usr/bin/env bash
# Verify clnrm plugins command output matches README claims
set -euo pipefail

CLNRM_BIN="${CLNRM_BIN:-/Users/sac/clnrm/target/release/clnrm}"

echo "🧪 Testing: clnrm plugins command"

# Run clnrm plugins
echo "▶️ Running: clnrm plugins"
OUTPUT=$("$CLNRM_BIN" plugins 2>&1)

echo "📤 Output:"
echo "$OUTPUT"

# Count production plugins
echo ""
echo "✅ Verification checks:"

PLUGIN_COUNT=0

if echo "$OUTPUT" | grep -q "generic_container"; then
    echo "  ✅ generic_container plugin found"
    ((PLUGIN_COUNT++))
else
    echo "  ❌ generic_container plugin NOT found"
    exit 1
fi

if echo "$OUTPUT" | grep -q "surreal_db"; then
    echo "  ✅ surreal_db plugin found"
    ((PLUGIN_COUNT++))
else
    echo "  ❌ surreal_db plugin NOT found"
    exit 1
fi

if echo "$OUTPUT" | grep -q "network_tools"; then
    echo "  ✅ network_tools plugin found"
    ((PLUGIN_COUNT++))
else
    echo "  ❌ network_tools plugin NOT found"
    exit 1
fi

if echo "$OUTPUT" | grep -q "ollama"; then
    echo "  ✅ ollama plugin found"
    ((PLUGIN_COUNT++))
else
    echo "  ❌ ollama plugin NOT found"
    exit 1
fi

if echo "$OUTPUT" | grep -q "vllm"; then
    echo "  ✅ vllm plugin found"
    ((PLUGIN_COUNT++))
else
    echo "  ❌ vllm plugin NOT found"
    exit 1
fi

if echo "$OUTPUT" | grep -q "tgi"; then
    echo "  ✅ tgi plugin found"
    ((PLUGIN_COUNT++))
else
    echo "  ❌ tgi plugin NOT found"
    exit 1
fi

# Check experimental plugins
EXPERIMENTAL_COUNT=0

if echo "$OUTPUT" | grep -q "chaos_engine"; then
    echo "  ✅ chaos_engine experimental plugin found"
    ((EXPERIMENTAL_COUNT++))
else
    echo "  ⚠️  chaos_engine experimental plugin NOT found"
fi

if echo "$OUTPUT" | grep -q "ai_test_generator"; then
    echo "  ✅ ai_test_generator experimental plugin found"
    ((EXPERIMENTAL_COUNT++))
else
    echo "  ⚠️  ai_test_generator experimental plugin NOT found"
fi

# Verify counts
echo ""
echo "📊 Plugin counts:"
echo "  Production plugins: $PLUGIN_COUNT (expected: 6)"
echo "  Experimental plugins: $EXPERIMENTAL_COUNT (expected: 2)"
echo "  Total: $((PLUGIN_COUNT + EXPERIMENTAL_COUNT)) (expected: 8)"

if [[ $PLUGIN_COUNT -eq 6 ]]; then
    echo "  ✅ Production plugin count matches README claim"
else
    echo "  ❌ Production plugin count MISMATCH (expected 6, got $PLUGIN_COUNT)"
    exit 1
fi

if [[ $EXPERIMENTAL_COUNT -eq 2 ]]; then
    echo "  ✅ Experimental plugin count matches README claim"
else
    echo "  ⚠️  Experimental plugin count mismatch (expected 2, got $EXPERIMENTAL_COUNT)"
fi

echo ""
echo "✅ README claims about 'clnrm plugins' verified successfully!"
