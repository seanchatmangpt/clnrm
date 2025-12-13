#!/bin/bash

# Test all examples to make sure they work

echo "🧪 Testing all examples..."
echo "========================================="

TOTAL=0
PASSED=0
FAILED=0

# Find all .clnrm.toml files
for file in $(find examples/ -name "*.clnrm.toml" | sort); do
    TOTAL=$((TOTAL + 1))

    echo -n "Testing $file ... "

    # Test validate
    if ./target/release/clnrm validate "$file" >/dev/null 2>&1; then
        echo "✅ PASS"
        PASSED=$((PASSED + 1))
    else
        echo "❌ FAIL"
        FAILED=$((FAILED + 1))
    fi
done

echo "========================================="
echo "📊 Results: $PASSED/$TOTAL passed, $FAILED failed"
echo "========================================="

if [ $FAILED -eq 0 ]; then
    echo "🎉 All examples working!"
    exit 0
else
    echo "❌ Some examples still broken"
    exit 1
fi
