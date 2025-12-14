#!/bin/bash
# Consistency Check Script
# Ensures code consistency across clnrm codebase
# Run in CI to prevent Mura (unevenness)

set -e

echo "🔍 Checking Code Consistency"
echo "============================"

# Check for consistent TODO comments (warn only, don't fail)
echo "📝 Checking TODO comment consistency..."
todo_count=$(grep -r "TODO:" crates/ | wc -l)
if [ "$todo_count" -gt 0 ]; then
    echo "⚠️  Found $todo_count TODO comments (should be actionable)"
    echo "   Format: 'TODO: <action> <details>'"
fi

# Check for consistent error handling
echo "🚨 Checking error handling consistency..."
if grep -r "unwrap()" crates/ | grep -v "test\|example"; then
    echo "❌ unwrap() found in production code. Use Result types instead."
    exit 1
fi

# Check for consistent documentation
echo "📚 Checking documentation consistency..."
missing_docs=$(find crates/ -name "*.rs" -exec grep -L "///" {} \; | wc -l)
if [ "$missing_docs" -gt 0 ]; then
    echo "⚠️  $missing_docs files missing documentation"
    # Don't fail CI for missing docs, just warn
fi

# Check for consistent import ordering
echo "📦 Checking import consistency..."
if grep -r "^use std::" crates/ | grep -A1 "^use crate::" | grep "^use std::"; then
    echo "❌ std imports should come before crate imports"
    exit 1
fi

echo "✅ Consistency checks passed!"
echo ""
echo "💡 Consistency Standards:"
echo "  - TODO comments: 'TODO: <action> <details>'"
echo "  - Error handling: Use Result<T, E>, no unwrap()"
echo "  - Documentation: Public functions should have /// docs"
echo "  - Imports: std::* before crate::*"