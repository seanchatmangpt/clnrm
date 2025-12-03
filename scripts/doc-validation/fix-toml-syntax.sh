#!/usr/bin/env bash
# Fix TOML syntax issues in documentation examples
# Issue: Dotted keys (attrs.all) cannot be used with inline tables in TOML

set -euo pipefail

echo "🔧 Fixing TOML syntax issues..."
echo ""

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# List of files with syntax issues
FAILED_FILES=(
  "examples/live-check/ci-cd.clnrm.toml"
  "examples/live-check/strict.clnrm.toml"
  "examples/live-check/basic.clnrm.toml"
  "examples/live-check/80-20.clnrm.toml"
  "examples/clnrm-case-study/tests/ai-production-readiness.clnrm.toml"
  "examples/clnrm-case-study/tests/ai-performance-benchmark.clnrm.toml"
  "examples/clnrm-case-study/tests/ai-character-interaction.clnrm.toml"
  "examples/optimus-prime-platform/optimus-prime-otel-validation.clnrm.toml"
  "examples/weaver-toml-configuration.clnrm.toml"
  "examples/case-studies/redteam-otlp-env.clnrm.toml"
  "examples/toml-config/rich-assertions-demo.toml"
  "examples/toml-config/complete-toml-demo.toml"
  "examples/templates/advanced-validators.clnrm.toml"
  "examples/templates/matrix-expansion.clnrm.toml"
  "examples/templates/ci-integration.clnrm.toml"
  "examples/templates/simple-variables.clnrm.toml"
  "examples/templates/macros-and-includes.clnrm.toml"
  "examples/readme-example-validation.clnrm.toml"
  "examples/template-workflow/otel-template-example.clnrm.toml"
)

FIXED=0

for file in "${FAILED_FILES[@]}"; do
  if [[ ! -f "$file" ]]; then
    echo "⚠️  File not found: $file"
    continue
  fi

  echo "📝 Fixing: $file"

  # Fix: Replace 'attrs.all = {' with proper table syntax
  # Pattern 1: attrs.all = { ... } on multiple lines
  # Pattern 2: attrs.any = { ... }
  # Pattern 3: Other dotted key patterns

  # Create backup
  cp "$file" "$file.bak"

  # Use perl for in-place editing with proper regex
  # Replace attrs.all = { ... } with [*.attrs.all] section
  perl -i -0pe 's/attrs\.all = \{([^}]+)\}/[attrs.all]\n$1/gs' "$file" || true
  perl -i -0pe 's/attrs\.any = \{([^}]+)\}/[attrs.any]\n$1/gs' "$file" || true

  # Alternative: Just comment out problematic lines and add note
  # This is safer than trying to fix automatically
  if ! python3 -c "import tomllib; tomllib.load(open('$file', 'rb'))" 2>/dev/null; then
    echo "  ⚠️  Auto-fix didn't work, adding comment"
    mv "$file.bak" "$file"

    # Add warning comment at top
    echo "# WARNING: This file has TOML syntax issues that need manual fixing" > "$file.tmp"
    echo "# Issue: Dotted keys (attrs.all) cannot be used with inline tables" >> "$file.tmp"
    echo "# See: https://toml.io/en/v1.0.0#inline-table" >> "$file.tmp"
    echo "" >> "$file.tmp"
    cat "$file" >> "$file.tmp"
    mv "$file.tmp" "$file"
  else
    FIXED=$((FIXED + 1))
    rm "$file.bak"
    echo "  ✅ Fixed successfully"
  fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Files processed: ${#FAILED_FILES[@]}"
echo "Successfully fixed: $FIXED"
echo "Needs manual fix: $((${#FAILED_FILES[@]} - FIXED))"
echo ""

if [[ $FIXED -eq ${#FAILED_FILES[@]} ]]; then
  echo "✅ All files fixed!"
else
  echo "⚠️  Some files need manual fixing"
  echo ""
  echo "Common TOML syntax rules:"
  echo "  ❌ attrs.all = { key = val }  # Dotted key + inline table"
  echo "  ✅ attrs = { all = { key = val } }  # Nested inline tables"
  echo "  ✅ [attrs.all]  # Table section"
  echo "     key = val"
fi
