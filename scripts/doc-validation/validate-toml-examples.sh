#!/usr/bin/env bash
# FM-005: Validate TOML examples in documentation
# Usage: ./scripts/doc-validation/validate-toml-examples.sh

set -euo pipefail

echo "🔍 FMEA FM-005: Validating TOML examples..."
echo "============================================"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Find all .toml files in docs/ and examples/
TOML_FILES=$(find docs examples -type f -name "*.toml" 2>/dev/null || true)

if [[ -z "$TOML_FILES" ]]; then
  echo "⚠️  No TOML files found in docs/ or examples/"
  echo "✅ PASS (no files to validate)"
  exit 0
fi

TOTAL=0
PASSED=0
FAILED=0

echo ""
echo "Validating TOML syntax..."
echo ""

for file in $TOML_FILES; do
  TOTAL=$((TOTAL + 1))
  echo "📄 Checking: $file"

  # Use a simple Python script to validate TOML syntax
  if python3 -c "import tomllib; tomllib.load(open('$file', 'rb'))" 2>/dev/null; then
    echo "  ✅ Valid TOML syntax"
    PASSED=$((PASSED + 1))
  else
    echo "  ❌ Invalid TOML syntax"
    # Show the error
    python3 -c "import tomllib; tomllib.load(open('$file', 'rb'))" 2>&1 | sed 's/^/    /' || true
    FAILED=$((FAILED + 1))
  fi
done

echo ""
echo "============================================"
echo "📊 TOML Validation Summary"
echo "============================================"
echo "Files checked: $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [[ $FAILED -eq 0 ]]; then
  echo "✅ SUCCESS: All TOML examples are syntactically valid!"
  exit 0
else
  echo "❌ FAILURE: Found $FAILED invalid TOML file(s)"
  echo ""
  echo "FMEA FM-005 mitigation: Fix TOML syntax before merging"
  exit 1
fi
