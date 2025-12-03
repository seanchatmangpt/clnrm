#!/usr/bin/env bash
# FM-002: Check for broken documentation links
# Usage: ./scripts/doc-validation/check-links.sh

set -euo pipefail

echo "🔍 FMEA FM-002: Checking documentation links..."
echo "================================================"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Find all markdown files
MARKDOWN_FILES=$(find . -type f -name "*.md" \
  -not -path "./target/*" \
  -not -path "./.git/*" \
  -not -path "./vendor/*" \
  -not -path "./node_modules/*")

TOTAL=0
BROKEN=0
CHECKED_FILES=0

echo ""
echo "Checking markdown files for broken links..."
echo ""

for file in $MARKDOWN_FILES; do
  CHECKED_FILES=$((CHECKED_FILES + 1))
  echo "📄 Checking: $file"

  # Extract markdown links: [text](path)
  LINKS=$(grep -oE '\]\([^)]+\)' "$file" | sed 's/](\(.*\))/\1/' || true)

  for link in $LINKS; do
    TOTAL=$((TOTAL + 1))

    # Skip external links (http://, https://, mailto:)
    if [[ "$link" =~ ^https?:// ]] || [[ "$link" =~ ^mailto: ]]; then
      continue
    fi

    # Remove anchor (#section)
    link_path="${link%%#*}"

    # Skip empty paths (just anchors)
    if [[ -z "$link_path" ]]; then
      continue
    fi

    # Resolve relative path
    base_dir="$(dirname "$file")"
    target="$base_dir/$link_path"

    # Normalize path (remove ./ and ../)
    target="$(cd "$base_dir" && realpath -m "$link_path" 2>/dev/null || echo "$target")"

    # Check if target exists
    if [[ ! -e "$target" ]]; then
      echo "  ❌ BROKEN: $link → $target"
      BROKEN=$((BROKEN + 1))
    fi
  done
done

echo ""
echo "================================================"
echo "📊 Link Check Summary"
echo "================================================"
echo "Files checked: $CHECKED_FILES"
echo "Links found: $TOTAL"
echo "Broken links: $BROKEN"
echo ""

if [[ $BROKEN -eq 0 ]]; then
  echo "✅ SUCCESS: All documentation links are valid!"
  exit 0
else
  echo "❌ FAILURE: Found $BROKEN broken link(s)"
  echo ""
  echo "FMEA FM-002 mitigation: Fix broken links before merging"
  exit 1
fi
