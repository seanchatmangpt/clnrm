#!/bin/bash
# End-to-end test for clnrm render command
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLNRM_BIN="${CLNRM_BIN:-cargo run -p clnrm --}"

echo "=== Testing clnrm render command ==="
echo

# Test 1: Basic rendering with --map flags
echo "Test 1: Basic rendering with --map flags"
$CLNRM_BIN render "$SCRIPT_DIR/example.toml.tera" --map svc=testapp --map env=production > /tmp/render_test1.toml
if grep -q "testapp_integration_test" /tmp/render_test1.toml && grep -q "production" /tmp/render_test1.toml; then
    echo "✓ Test 1 PASSED"
else
    echo "✗ Test 1 FAILED"
    exit 1
fi
echo

# Test 2: Rendering with --output flag
echo "Test 2: Rendering with --output flag"
$CLNRM_BIN render "$SCRIPT_DIR/example.toml.tera" --map svc=api --map env=staging -o /tmp/render_test2.toml
if [ -f /tmp/render_test2.toml ] && grep -q "api_integration_test" /tmp/render_test2.toml; then
    echo "✓ Test 2 PASSED"
else
    echo "✗ Test 2 FAILED"
    exit 1
fi
echo

# Test 3: Rendering with --show-vars
echo "Test 3: Rendering with --show-vars"
OUTPUT=$($CLNRM_BIN render "$SCRIPT_DIR/example.toml.tera" --map svc=web --map env=dev --show-vars 2>&1)
if echo "$OUTPUT" | grep -q "svc = \"web\"" && echo "$OUTPUT" | grep -q "env = \"dev\""; then
    echo "✓ Test 3 PASSED"
else
    echo "✗ Test 3 FAILED"
    exit 1
fi
echo

# Test 4: Invalid mapping format should fail
echo "Test 4: Invalid mapping format should fail"
if $CLNRM_BIN render "$SCRIPT_DIR/example.toml.tera" --map invalid_no_equals 2>&1 | grep -q "expected key=value format"; then
    echo "✓ Test 4 PASSED"
else
    echo "✗ Test 4 FAILED"
    exit 1
fi
echo

# Test 5: Multiple variables in single command
echo "Test 5: Multiple variables in single command"
$CLNRM_BIN render "$SCRIPT_DIR/example.toml.tera" \
    --map svc=myservice \
    --map env=uat \
    -o /tmp/render_test5.toml
if grep -q "myservice_integration_test" /tmp/render_test5.toml && grep -q "uat" /tmp/render_test5.toml; then
    echo "✓ Test 5 PASSED"
else
    echo "✗ Test 5 FAILED"
    exit 1
fi
echo

echo "=== All render tests PASSED ==="

# Cleanup
rm -f /tmp/render_test*.toml
