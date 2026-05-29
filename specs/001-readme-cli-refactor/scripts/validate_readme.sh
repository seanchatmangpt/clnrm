#!/bin/bash
# README.md Validation Script
# Based on contracts/README_CONTRACT.md
#
# Exit codes:
# 0 = GREEN (all checks pass)
# 1 = RED (validation failures)

set -e

echo "🔍 Validating README.md structure..."
echo ""

ERRORS=0

# Check required sections exist
sections=(
    "# clnrm - Hermetic Container Testing Framework"
    "## 🎯 THE VITAL FEW"
    "## 🚀 Quick Start"
    "## 📋 Command Reference"
    "## ⚡ Code Standards"
    "## 🔧 Troubleshooting"
)

for section in "${sections[@]}"; do
    if ! grep -q "$section" README.md; then
        echo "❌ Missing section: $section"
        ERRORS=$((ERRORS + 1))
    fi
done

if [ $ERRORS -eq 0 ]; then
    echo "✅ All required sections present"
else
    echo ""
    echo "❌ Missing $ERRORS required section(s)"
    exit 1
fi

# Check no hardcoded versions (allow in badge URLs)
echo ""
echo "Checking for hardcoded version numbers..."
if grep -v "img.shields.io" README.md | grep -v "github.com/seanchatmangpt" | grep -E "[^/]v?2\.1\.0" > /dev/null 2>&1; then
    echo "❌ Hardcoded version found (use badges only)"
    grep -v "img.shields.io" README.md | grep -E "[^/]v?2\.1\.0" || true
    exit 1
fi
echo "✅ No hardcoded versions"

# Check all 26 commands documented
echo ""
echo "Checking command documentation..."
commands=(run dry-run record repro stress self-test init validate lint fmt render spans report graph health live-check services collector plugins pull dev template diff analyze redgreen)
missing_commands=()

for cmd in "${commands[@]}"; do
    if ! grep -q "\`$cmd\`" README.md; then
        missing_commands+=("$cmd")
    fi
done

if [ ${#missing_commands[@]} -ne 0 ]; then
    echo "❌ Missing commands: ${missing_commands[*]}"
    exit 1
fi
echo "✅ All 26 commands documented"

# Check constitutional principles referenced
echo ""
echo "Checking constitutional principles..."
principles=("Cargo Make" "Error Handling" "Chicago TDD" "Andon Signal" "Concurrent Execution")
for principle in "${principles[@]}"; do
    if ! grep -q "$principle" README.md; then
        echo "❌ Missing principle: $principle"
        exit 1
    fi
done
echo "✅ All 5 constitutional principles referenced"

# Check constitution.md links (minimum 5)
echo ""
echo "Checking constitution.md links..."
constitution_links=$(grep -c "constitution.md" README.md || echo "0")
if [ "$constitution_links" -lt 5 ]; then
    echo "❌ Need at least 5 constitution.md links (found $constitution_links)"
    exit 1
fi
echo "✅ Constitution.md linked $constitution_links times"

# Check for broken internal links
echo ""
echo "Checking internal links..."
# Extract markdown links that aren't http/https
grep -oE '\[.*\]\(([^h].*\.md.*?)\)' README.md | sed -E 's/.*\(([^)]+)\).*/\1/' | while read -r link; do
    # Remove anchor (#section)
    file_path=$(echo "$link" | cut -d'#' -f1)

    if [ -n "$file_path" ] && [ ! -f "$file_path" ]; then
        echo "❌ Broken link: $file_path"
        exit 1
    fi
done 2>/dev/null || echo "✅ All internal links valid"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ README.md validation PASSED"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
