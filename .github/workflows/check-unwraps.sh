#!/bin/bash
# Unwrap Detection Script
# Scans production code for .unwrap() and .expect() calls
# Excludes test modules (after #[cfg(test)])

set -uo pipefail

echo "🔍 Scanning for .unwrap() and .expect() in production code..."
echo ""
echo "Checking: crates/clnrm-core/src/ (excluding tests)"
echo "=========================================="

FOUND_UNWRAPS=0

for file in $(find crates/clnrm-core/src -name "*.rs" -type f | grep -v "/tests/"); do
  # Find the line number where #[cfg(test)] starts (if any)
  TEST_START=$(grep -n "^#\[cfg(test)\]" "$file" | head -1 | cut -d: -f1)

  if [ -n "$TEST_START" ]; then
    # Only check lines before the test module
    UNWRAPS=$(head -n $((TEST_START - 1)) "$file" | grep -n '\.unwrap()\|\.expect(' | grep -v '// OK: ' | grep -v '^[0-9]*:[[:space:]]*//' || true)
  else
    # No test module, check entire file
    UNWRAPS=$(grep -n '\.unwrap()\|\.expect(' "$file" | grep -v '// OK: ' | grep -v '^[0-9]*:[[:space:]]*//' || true)
  fi

  if [ -n "$UNWRAPS" ]; then
    echo "❌ Found unwraps in: $file"
    echo "$UNWRAPS" | sed 's/^/   Line /'
    FOUND_UNWRAPS=1
  fi
done

echo ""

if [ $FOUND_UNWRAPS -eq 1 ]; then
  echo "🚨 CRITICAL: Production code MUST NOT use .unwrap() or .expect()"
  echo "   Use proper Result<T, CleanroomError> error handling instead."
  echo ""
  echo "If this is a false positive (e.g., in const initialization), add comment:"
  echo "   // OK: Safe unwrap - <reason>"
  exit 1
else
  echo "✅ SUCCESS: No unwraps/expects found in production code!"
  exit 0
fi
