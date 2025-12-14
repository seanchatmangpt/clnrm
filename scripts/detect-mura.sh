#!/bin/bash
# Detect Mura (inconsistency) patterns in the codebase
# This script identifies various forms of inconsistency that violate standards

set -e

echo "🔍 Detecting Mura (inconsistency) patterns..."
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

ERRORS=0
WARNINGS=0

# Function to report issues
report_error() {
    echo -e "${RED}❌ ERROR: $1${NC}"
    ((ERRORS++))
}

report_warning() {
    echo -e "${YELLOW}⚠️  WARNING: $1${NC}"
    ((WARNINGS++))
}

report_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# 1. Check formatting consistency
echo ""
echo "1. 📏 Checking code formatting consistency..."
if cargo fmt --check --quiet 2>/dev/null; then
    report_success "All code is properly formatted"
else
    VIOLATIONS=$(cargo fmt --check 2>&1 | wc -l)
    report_error "Code formatting violations found: $VIOLATIONS lines need formatting"
fi

# 2. Check import organization consistency
echo ""
echo "2. 📦 Checking import organization consistency..."

# Find files with inconsistent import patterns
INCONSISTENT_FILES=$(find crates/clnrm-core/src -name "*.rs" -exec awk '
BEGIN { in_use_section = 0; inconsistent = 0 }
/^use / { in_use_section = 1 }
/^$/ { if (in_use_section) in_use_section = 0 }
/^[^u]/ { if (in_use_section) inconsistent = 1 }
END { if (inconsistent) print FILENAME }
' {} \; | wc -l)

if [ "$INCONSISTENT_FILES" -gt 0 ]; then
    report_warning "Found $INCONSISTENT_FILES files with potentially inconsistent import organization"
else
    report_success "Import organization appears consistent"
fi

# 3. Check unwrap() usage in production code
echo ""
echo "3. 🚫 Checking unwrap() usage in production code..."

# Count unwrap() calls in production code (excluding tests)
UNWRAP_COUNT=$(find crates/clnrm-core/src -name "*.rs" -exec grep -l "\.unwrap()" {} \; | xargs grep -h "\.unwrap()" | grep -v "#\[test\]" | grep -v "///" | grep -v "//" | wc -l)

if [ "$UNWRAP_COUNT" -gt 0 ]; then
    report_error "Found $UNWRAP_COUNT unwrap() calls in production code (should use Result handling)"
else
    report_success "No unwrap() calls found in production code"
fi

# 4. Check test coverage consistency
echo ""
echo "4. 🧪 Checking test coverage consistency..."

# Count modules with vs without tests
TOTAL_MODULES=$(find crates/clnrm-core/src -type d | grep -v "/tests" | wc -l)
MODULES_WITH_TESTS=$(find crates/clnrm-core/src -type d | grep -v "/tests" | while read dir; do if [ -f "$dir/mod.rs" ] || [ -f "$dir/lib.rs" ]; then ls -la "$dir"/*test*.rs 2>/dev/null | head -1 >/dev/null && echo "$dir"; fi; done | wc -l)

if [ "$TOTAL_MODULES" -gt 0 ]; then
    COVERAGE_PERCENT=$((MODULES_WITH_TESTS * 100 / TOTAL_MODULES))
    if [ "$COVERAGE_PERCENT" -lt 80 ]; then
        report_warning "Test coverage: $COVERAGE_PERCENT% ($MODULES_WITH_TESTS/$TOTAL_MODULES modules have tests)"
    else
        report_success "Test coverage: $COVERAGE_PERCENT% ($MODULES_WITH_TESTS/$TOTAL_MODULES modules have tests)"
    fi
fi

# 5. Check documentation consistency
echo ""
echo "5. 📚 Checking documentation consistency..."

# Count files with vs without module docs
TOTAL_RS_FILES=$(find crates/clnrm-core/src -name "*.rs" | wc -l)
DOCS_COUNT=$(find crates/clnrm-core/src -name "*.rs" -exec head -5 {} \; | grep -c "^//!")

if [ "$TOTAL_RS_FILES" -gt 0 ]; then
    DOCS_PERCENT=$((DOCS_COUNT * 100 / TOTAL_RS_FILES))
    if [ "$DOCS_PERCENT" -lt 50 ]; then
        report_warning "Documentation coverage: $DOCS_PERCENT% ($DOCS_COUNT/$TOTAL_RS_FILES files have module docs)"
    else
        report_success "Documentation coverage: $DOCS_PERCENT% ($DOCS_COUNT/$TOTAL_RS_FILES files have module docs)"
    fi
fi

# 6. Check for TODO/FIXME comments (incomplete work)
echo ""
echo "6. 📝 Checking for incomplete work markers..."

TODO_COUNT=$(find crates/clnrm-core/src -name "*.rs" -exec grep -l "TODO\|FIXME" {} \; | wc -l)
if [ "$TODO_COUNT" -gt 0 ]; then
    report_warning "Found $TODO_COUNT files with TODO/FIXME comments (incomplete work)"
else
    report_success "No TODO/FIXME comments found"
fi

# 7. Check for inconsistent error handling patterns
echo ""
echo "7. 🛡️  Checking error handling consistency..."

# Look for mixed error handling patterns
MIXED_PATTERNS=$(find crates/clnrm-core/src -name "*.rs" -exec awk '
/Result.*CleanroomError/ { result_count++ }
/unwrap\(\)/ { unwrap_count++ }
END {
    if (result_count > 0 && unwrap_count > 0) {
        print FILENAME
    }
}
' {} \; | wc -l)

if [ "$MIXED_PATTERNS" -gt 0 ]; then
    report_warning "Found $MIXED_PATTERNS files mixing Result and unwrap() patterns"
else
    report_success "Error handling patterns are consistent"
fi

# Summary
echo ""
echo "=============================================="
echo "🎯 Mura Detection Summary:"
echo "Errors: $ERRORS"
echo "Warnings: $WARNINGS"

if [ "$ERRORS" -eq 0 ] && [ "$WARNINGS" -eq 0 ]; then
    echo -e "${GREEN}✅ No Mura detected! Codebase is consistent.${NC}"
    exit 0
elif [ "$ERRORS" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Only warnings found. Address for better consistency.${NC}"
    exit 0
else
    echo -e "${RED}❌ Critical inconsistencies found. Must be addressed.${NC}"
    exit 1
fi